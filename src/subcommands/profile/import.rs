use std::{env::current_dir, path::PathBuf};

use anyhow::{bail, Result};
use colored::Colorize;
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

#[derive(clap::Args, Clone, Debug)]
/// Import an existing profile
pub struct Args {
    /// The name of the profile
    #[clap(long, short)]
    pub name: Option<String>,
    /// The path to the profile
    #[clap(long, short)]
    #[clap(value_hint(clap::ValueHint::FilePath))]
    pub path: Option<PathBuf>,
    /// The `.minecraft` directory the profile will output mods and other files to
    #[clap(long, short)]
    #[clap(value_hint(clap::ValueHint::DirPath))]
    pub minecraft_dir: Option<PathBuf>,
    /// Whether or not to embed the profile,
    /// i.e not make a file for it and instead store it directly in the ferium/ogj-config.toml
    #[clap(long, short)]
    pub embed: bool,
}

pub async fn import(
    config: &mut Config,
    Args {
        name,
        path,
        minecraft_dir,
        embed,
    }: Args,
) -> Result<()> {
    println!(
        "{}",
        "Don't import profiles from people you don't trust!"
            .yellow()
            .bold()
    );

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

    let minecraft_dir = match minecraft_dir {
        Some(minecraft_dir) => minecraft_dir,
        None => ask_minecraft_dir().await?,
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
        minecraft_dir,
    ));

    config.active_profile = config.profiles.len() - 1;

    Ok(())
}

async fn ask_minecraft_dir() -> Result<PathBuf> {
    let mut selected_mods_dir = get_minecraft_dir();
    println!(
        "The default `.minecraft` directory is {}",
        selected_mods_dir.display()
    );
    if Confirm::new("Would you like to specify a custom `.minecraft` directory?")
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
