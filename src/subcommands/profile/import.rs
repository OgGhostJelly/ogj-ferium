use std::{env::current_dir, path::PathBuf};

use anyhow::{bail, Result};
use inquire::{
    validator::{ErrorMessage, Validation},
    Confirm, Text,
};
use libium::{
    config::{
        self,
        structs::{Config, ProfileItem, ProfileSource},
    },
    get_minecraft_dir,
    iter_ext::IterExt,
};

use crate::file_picker::{pick_file, pick_folder};

use super::check_output_directory;

pub async fn import(
    config: &mut Config,
    name: Option<String>,
    path: Option<PathBuf>,
    mods_dir: Option<PathBuf>,
    resourcepacks_dir: Option<PathBuf>,
    shaderpacks_dir: Option<PathBuf>,
    embed: bool,
) -> Result<()> {
    let path = if let Some(path) = path {
        path
    } else {
        println!("Where is the profile to import?");
        if let Some(path) = pick_file(current_dir()?, "Pick the profile to import", "Profile")? {
            path
        } else {
            bail!("Please select a path to a profile.");
        }
    }
    .canonicalize()?;

    let Some(profile) = config::read_profile(&path)? else {
        bail!("No profile was found at the given path.");
    };

    let mods_dir = match mods_dir {
        Some(mods_dir) => mods_dir,
        None => get_dir("mods").await?,
    };

    let resourcepacks_dir = match resourcepacks_dir {
        Some(resourcepacks_dir) => resourcepacks_dir,
        None => get_dir("resourcepacks").await?,
    };

    let shaderpacks_dir = match shaderpacks_dir {
        Some(shaderpacks_dir) => shaderpacks_dir,
        None => get_dir("shaderpacks").await?,
    };

    let name = if let Some(name) = name {
        name
    } else {
        let profiles = config
            .profiles
            .iter()
            .map(|item| item.config.name.clone())
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
        name
    };

    config.profiles.push(ProfileItem::new(
        if embed {
            ProfileSource::Embedded(Box::new(profile))
        } else {
            ProfileSource::Path(path)
        },
        name,
        mods_dir,
        shaderpacks_dir,
        resourcepacks_dir,
    ));

    Ok(())
}

async fn get_dir(dir: &str) -> Result<PathBuf> {
    let mut selected_mods_dir = get_minecraft_dir().join(dir);
    println!(
        "The default {dir} directory is {}",
        selected_mods_dir.display()
    );
    if Confirm::new(&format!(
        "Would you like to specify a custom {dir} directory?"
    ))
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
        }
    }
    Ok(selected_mods_dir)
}
