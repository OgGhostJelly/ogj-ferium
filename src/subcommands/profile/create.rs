use super::{check_output_directory, pick_minecraft_versions, pick_mod_loader};
use crate::{file_picker::pick_folder, try_iter_profiles};
use anyhow::{bail, ensure, Context as _, Result};
use colored::Colorize as _;
use inquire::{
    validator::{ErrorMessage, Validation},
    Confirm, Select, Text,
};
use libium::{
    config::{self, structs::{Config, ModLoader, Profile}},
    get_minecraft_dir,
};
use std::{env::current_dir, path::PathBuf};

#[expect(clippy::option_option)]
pub async fn create(
    config: &mut Config,
    import: Option<Option<String>>,
    game_versions: Option<Vec<String>>,
    mod_loader: Option<ModLoader>,
    name: Option<String>,
    output_dir: Option<PathBuf>,
) -> Result<()> {
    let mut profile = match (game_versions, mod_loader, name, output_dir) {
        (Some(game_versions), Some(mod_loader), Some(name), output_dir) => {
            for (_, profile) in try_iter_profiles(&config.profiles) {
                ensure!(
                    !profile.name.eq_ignore_ascii_case(&name),
                    "A profile with name {name} already exists"
                );
            }
            let output_dir = output_dir.unwrap_or_else(|| get_minecraft_dir().join("mods"));
            ensure!(
                output_dir.is_absolute(),
                "The provided output directory is not absolute, i.e. it is a relative path"
            );

            Profile::new(name, output_dir, game_versions, mod_loader)
        }
        (None, None, None, None) => {
            let mut selected_mods_dir = get_minecraft_dir().join("mods");
            println!(
                "The default mods directory is {}",
                selected_mods_dir.display()
            );
            if Confirm::new("Would you like to specify a custom mods directory?")
                .prompt()
                .unwrap_or_default()
            {
                if let Some(dir) = pick_folder(
                    &selected_mods_dir,
                    "Pick an output directory",
                    "Output Directory",
                )? {
                    check_output_directory(&dir).await?;
                    selected_mods_dir = dir;
                };
            }

            let profiles = config.profiles.clone();
            let name = Text::new("What should this profile be called")
                .with_validator(move |s: &str| {
                    Ok(if try_iter_profiles(&profiles)
                    .any(|(_, p)| p.name.eq_ignore_ascii_case(s)) {
                        Validation::Invalid(ErrorMessage::Custom(
                            "A profile with that name already exists".to_owned(),
                        ))
                    } else {
                        Validation::Valid
                    })
                })
                .prompt()?;

            Profile::new(
                name,
                selected_mods_dir,
                pick_minecraft_versions(&[]).await?,
                pick_mod_loader(None)?,
            )
        }
        _ => {
            bail!("Provide the name, game version, mod loader, and output directory options to create a profile")
        }
    };

    if let Some(from) = import {
        ensure!(
            !config.profiles.is_empty(),
            "There are no profiles configured to import mods from"
        );

        // If the profile name has been provided as an option
        if let Some(profile_name) = from {
            let (_, mut import_profile) = try_iter_profiles(&config.profiles)
                .find(|(_, profile)| profile.name.eq_ignore_ascii_case(&profile_name))
                .context("The profile name provided does not exist")?;
            profile.mods.append(&mut import_profile.mods);
        } else {
            let mut profile_names = vec![];
            let mut profiles = vec![];

            for (_, profile) in try_iter_profiles(&config.profiles) {
                profile_names.push(profile.name.clone());
                profiles.push(profile);
            }
            if let Ok(selection) =
                Select::new("Select which profile to import mods from", profile_names)
                    .with_starting_cursor(config.active_profile)
                    .raw_prompt()
            {
                let import_profile = &mut profiles[selection.index];
                profile
                    .mods
                    .append(&mut import_profile.mods);
            }
        };
    }

    println!(
        "{}",
        "After adding your mods, remember to run `ferium upgrade` to download them!".yellow()
    );

    let mut path = profile.name.clone();
    if !path.ends_with(".json") {
        path.push_str(".json");
    }
    let path = current_dir()?.join(path);
    config::write_profile(&path, &profile)?;
    config.profiles.push(path);
    config.active_profile = config.profiles.len() - 1; // Make created profile active
    Ok(())
}
