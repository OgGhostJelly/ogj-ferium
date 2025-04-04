use super::{check_output_directory, pick_minecraft_version, pick_mod_loader};
use crate::{file_picker::pick_folder, try_iter_profiles};
use anyhow::{bail, ensure, Context as _, Result};
use colored::Colorize as _;
use inquire::{
    validator::{ErrorMessage, Validation},
    Confirm, Select, Text,
};
use libium::{
    config::{
        self,
        structs::{Config, ModLoader, Profile, ProfileItem, ProfileSource, Version},
    },
    get_minecraft_dir,
    iter_ext::IterExt,
};
use std::path::PathBuf;

#[expect(clippy::option_option, clippy::too_many_arguments)]
pub async fn create(
    config: &mut Config,
    import: Option<Option<String>>,
    game_versions: Option<Vec<Version>>,
    mod_loader: Option<ModLoader>,
    name: Option<String>,
    mods_dir: Option<PathBuf>,
    resourcepacks_dir: Option<PathBuf>,
    shaderpacks_dir: Option<PathBuf>,
    profile_path: Option<PathBuf>,
    embed: bool,
) -> Result<()> {
    let item = match (
        game_versions,
        mod_loader,
        name,
        mods_dir,
        resourcepacks_dir,
        shaderpacks_dir,
    ) {
        (
            Some(game_versions),
            Some(mod_loader),
            Some(name),
            mods_dir,
            resourcepacks_dir,
            shaderpacks_dir,
        ) => {
            for item in &config.profiles {
                ensure!(
                    !item.name.eq_ignore_ascii_case(&name),
                    "A profile with that name already exists"
                );
            }
            let mods_dir = mods_dir.unwrap_or_else(|| get_minecraft_dir().join("mods"));
            ensure!(
                mods_dir.is_absolute(),
                "The provided mods directory is not absolute, i.e. it is a relative path"
            );
            let resourcepacks_dir =
                resourcepacks_dir.unwrap_or_else(|| get_minecraft_dir().join("resourcepacks"));
            ensure!(
                mods_dir.is_absolute(),
                "The provided resourcepacks directory is not absolute, i.e. it is a relative path"
            );
            let shaderpacks_dir =
                shaderpacks_dir.unwrap_or_else(|| get_minecraft_dir().join("shaderpacks"));
            ensure!(
                mods_dir.is_absolute(),
                "The provided shaderpacks directory is not absolute, i.e. it is a relative path"
            );

            let mut profile = Profile::new(Some(game_versions), mod_loader);

            import_from(config, import, &mut profile)?;

            if embed {
                ProfileItem::new(
                    ProfileSource::Embedded(Box::new(profile)),
                    name,
                    mods_dir,
                    shaderpacks_dir,
                    resourcepacks_dir,
                )
            } else {
                let path = ProfileItem::infer_path(profile_path, &name)?;
                config::write_profile(&path, &profile)?;
                ProfileItem::new(
                    ProfileSource::Path(path),
                    name,
                    mods_dir,
                    shaderpacks_dir,
                    resourcepacks_dir,
                )
            }
        }
        (None, None, None, None, None, None) => {
            async fn get_dir(dir: &str) -> Result<PathBuf> {
                let mut selected_dir = get_minecraft_dir().join(dir);
                println!("The default {dir} directory is {}", selected_dir.display());
                if Confirm::new(&format!(
                    "Would you like to specify a custom {dir} directory?"
                ))
                .prompt()
                .unwrap_or_default()
                {
                    if let Some(dir) = pick_folder(
                        &selected_dir,
                        "Pick an output directory",
                        "Output Directory",
                    )? {
                        check_output_directory(&dir).await?;
                        selected_dir = dir;
                    }
                }
                Ok(selected_dir)
            }

            let mods_dir = get_dir("mods").await?;
            let resourcepacks_dir = get_dir("resourcepacks").await?;
            let shaderpacks_dir = get_dir("shaderpacks").await?;

            let profiles = config
                .profiles
                .iter()
                .map(|item| item.name.clone())
                .collect_vec();
            let name = Text::new("What should this profile be called")
                .with_validator(move |s: &str| {
                    Ok(
                        if profiles.iter().any(|name| name.eq_ignore_ascii_case(s)) {
                            Validation::Invalid(ErrorMessage::Custom(
                                "A profile with that name already exists".to_owned(),
                            ))
                        } else {
                            Validation::Valid
                        },
                    )
                })
                .prompt()?;

            let mut profile = Profile::new(
                Some(pick_minecraft_version(&[]).await?),
                pick_mod_loader(None)?,
            );

            import_from(config, import, &mut profile)?;

            if embed {
                ProfileItem::new(
                    ProfileSource::Embedded(Box::new(profile)),
                    name,
                    mods_dir,
                    shaderpacks_dir,
                    resourcepacks_dir,
                )
            } else {
                let path = ProfileItem::infer_path(profile_path, &name)?;
                config::write_profile(&path, &profile)?;
                ProfileItem::new(
                    ProfileSource::Path(path),
                    name,
                    mods_dir,
                    shaderpacks_dir,
                    resourcepacks_dir,
                )
            }
        }
        _ => {
            bail!("Provide the name, game version, mod loader, and output directory options to create a profile")
        }
    };

    println!(
        "{}",
        "After adding your mods, remember to run `ferium upgrade` to download them!".yellow()
    );

    config.profiles.push(item);
    config.active_profile = config.profiles.len() - 1; // Make created profile active
    Ok(())
}

#[expect(clippy::option_option)]
fn import_from(
    config: &mut Config,
    import: Option<Option<String>>,
    profile: &mut Profile,
) -> Result<()> {
    if let Some(from) = import {
        ensure!(
            !config.profiles.is_empty(),
            "There are no profiles configured to import mods from"
        );

        // If the profile name has been provided as an option
        if let Some(profile_name) = from {
            let (_, import_profile) = try_iter_profiles(&mut config.profiles)
                .find(|(item, _)| item.name.eq_ignore_ascii_case(&profile_name))
                .context("The profile name provided does not exist")?;
            for (name, source) in import_profile.mods {
                profile.mods.insert(name, source);
            }
            for (name, source) in import_profile.resourcepacks {
                profile.resourcepacks.insert(name, source);
            }
            for (name, source) in import_profile.shaders {
                profile.shaders.insert(name, source);
            }
            for (name, source) in import_profile.modpacks {
                profile.modpacks.insert(name, source);
            }
        } else {
            let mut profile_names = vec![];
            let mut profiles = vec![];

            for (item, profile) in try_iter_profiles(&mut config.profiles) {
                profile_names.push(item.name.clone());
                profiles.push(profile);
            }
            if let Ok(selection) =
                Select::new("Select which profile to import mods from", profile_names)
                    .with_starting_cursor(config.active_profile)
                    .raw_prompt()
            {
                let import_profile = profiles.swap_remove(selection.index);
                for (name, source) in import_profile.mods {
                    profile.mods.insert(name, source);
                }
                for (name, source) in import_profile.resourcepacks {
                    profile.resourcepacks.insert(name, source);
                }
                for (name, source) in import_profile.shaders {
                    profile.shaders.insert(name, source);
                }
                for (name, source) in import_profile.modpacks {
                    profile.modpacks.insert(name, source);
                }
            }
        }
    }
    Ok(())
}
