use std::{
    env::{current_dir, var_os},
    path::PathBuf,
    sync::LazyLock,
};

use anyhow::{bail, Result};
use colored::Colorize;
use inquire::Confirm;
use libium::HOME;

use crate::file_picker::pick_file;

pub async fn migrate(old_config_path: Option<PathBuf>, force: bool) -> Result<()> {
    if !force {
        println!(
            "{}",
            "This will overwrite your existing ogj config".yellow()
        );
        if !Confirm::new("Do you want to continue?")
            .prompt()
            .unwrap_or_default()
        {
            return Ok(());
        }
    }

    let old_config_path = match old_config_path {
        Some(path) => path,
        None => match var_os("FERIUM_CONFIG_FILE").map(Into::into) {
            Some(path) => path,
            None => get_old_default_config_path()?,
        },
    };

    libium::config::migrate_legacy_config(&old_config_path)?;

    Ok(())
}

fn get_old_default_config_path() -> Result<PathBuf> {
    if OLD_DEFAULT_CONFIG_PATH.exists() {
        Ok(OLD_DEFAULT_CONFIG_PATH.clone())
    } else {
        println!("Where is the old config to migrate?");
        if let Some(path) = pick_file(
            current_dir()?,
            "Pick the config to migrate",
            "Ferium Config",
        )? {
            Ok(path)
        } else {
            bail!("Please select a path to a config.");
        }
    }
}

static OLD_DEFAULT_CONFIG_PATH: LazyLock<PathBuf> =
    LazyLock::new(|| HOME.join(".config").join("ferium").join("config.json"));
