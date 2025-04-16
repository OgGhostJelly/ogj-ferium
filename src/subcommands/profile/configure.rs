use super::{check_output_directory, pick_minecraft_version, pick_mod_loader};
use crate::{file_picker::pick_folder, get_active_profile};
use anyhow::Result;
use inquire::{Select, Text};
use libium::{
    config::structs::{Config, ModLoader, Version},
    iter_ext::IterExt,
};
use std::path::PathBuf;

/// Configure the current profile's name, Minecraft version, mod loader, and output directory.
/// Optionally, provide the settings to change as arguments.
#[derive(clap::Args, Clone, Debug)]
#[clap(visible_aliases = ["config", "conf"])]
pub struct Args {
    /// The Minecraft version(s) to consider as compatible
    #[clap(long, short = 'v')]
    game_versions: Vec<Version>,
    /// The mod loader(s) to consider as compatible
    #[clap(long, short = 'l')]
    #[clap(value_enum)]
    mod_loaders: Vec<ModLoader>,
    /// The name of the profile
    #[clap(long, short)]
    name: Option<String>,
    /// The .minecraft directory
    #[clap(long, short)]
    #[clap(value_hint(clap::ValueHint::DirPath))]
    minecraft_dir: Option<PathBuf>,
}

pub async fn configure(
    config: &mut Config,
    Args {
        game_versions,
        mod_loaders,
        name,
        minecraft_dir,
    }: Args,
) -> Result<()> {
    let (profile_item, mut profile) = get_active_profile(config)?;

    let mut interactive = true;

    if !game_versions.is_empty() {
        profile.filters.versions = Some(game_versions);
        interactive = false;
    }
    if !mod_loaders.is_empty() {
        profile.filters.mod_loaders = Some(mod_loaders);
        interactive = false;
    }
    if let Some(name) = name {
        profile_item.name = name;
        interactive = false;
    }
    if let Some(minecraft_dir) = minecraft_dir {
        profile_item.minecraft_dir = minecraft_dir;
        interactive = false;
    }

    if interactive {
        let items = vec![
            // Show a file dialog
            "Minecraft directory",
            // Show a picker of Minecraft versions to select from
            "Minecraft version",
            // Show a picker to change mod loader
            "Mod loader",
            // Show a dialog to change name
            "Profile Name",
            // Quit the configuration
            "Quit",
        ];

        while let Ok(selection) =
            Select::new("Which setting would you like to change", items.clone()).raw_prompt()
        {
            match selection.index {
                0 => {
                    if let Some(dir) = pick_folder(
                        &profile_item.minecraft_dir,
                        "Pick an output directory",
                        "Output Directory",
                    )? {
                        check_output_directory(&dir).await?;
                        profile_item.minecraft_dir = dir;
                    }
                }
                1 => {
                    let versions = profile
                        .filters
                        .versions
                        .as_ref()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(std::string::ToString::to_string)
                        .collect_vec();

                    if let Ok(selection) = pick_minecraft_version(&versions).await {
                        profile.filters.versions = Some(selection);
                    }
                }
                2 => {
                    if let Ok(selection) = pick_mod_loader(
                        profile.filters.mod_loaders.as_ref().and_then(|x| x.first()),
                    ) {
                        profile.filters.mod_loaders = match selection {
                            ModLoader::Quilt => Some(vec![ModLoader::Quilt, ModLoader::Fabric]),
                            loader => Some(vec![loader]),
                        }
                    }
                }
                3 => {
                    if let Ok(new_name) = Text::new("Change the profile's name")
                        .with_default(&profile_item.name)
                        .prompt()
                    {
                        profile_item.name = new_name;
                    } else {
                        continue;
                    }
                }
                4 => break,
                _ => unreachable!(),
            }
            println!();
        }
    }

    profile.write()?;

    Ok(())
}
