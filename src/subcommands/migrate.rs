use std::{env::current_dir, path::PathBuf};

use anyhow::{bail, Result};
use colored::Colorize;
use inquire::Confirm;

use crate::file_picker::pick_file;

pub async fn migrate(old_config_path: Option<PathBuf>, force: bool) -> Result<()> {
    if !force {
        println!(
            "{}",
            "This will overwrite your existing ogj config".yellow()
        );
        if !Confirm::new(&format!("Do you want to continue?"))
            .prompt()
            .unwrap_or_default()
        {
            return Ok(());
        }
    }

    let old_config_path = if let Some(path) = old_config_path {
        path
    } else {
        println!("Where is the old config to migrate?");
        if let Some(path) = pick_file(
            current_dir()?,
            "Pick the config to migrate",
            "Ferium Config",
        )? {
            path
        } else {
            bail!("Please select a path to a config.");
        }
    };

    libium::config::migrate_legacy_config(&old_config_path)?;

    Ok(())
}
