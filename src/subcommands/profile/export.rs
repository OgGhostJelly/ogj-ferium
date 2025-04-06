use std::path::PathBuf;

use anyhow::{bail, Result};
use colored::Colorize;
use libium::config::{
    structs::{Config, ProfileSource},
    write_profile,
};

use crate::get_active_profile;

pub async fn export(config: &mut Config, output_path: PathBuf, name: Option<String>) -> Result<()> {
    let profile = if let Some(name) = name {
        let Some(item) = config
            .profiles
            .iter()
            .find(|item| item.config.name.eq_ignore_ascii_case(&name))
        else {
            bail!("A profile with that name doesnt exist");
        };

        let path = match &item.profile {
            ProfileSource::Path(path) => path.display().to_string().blue().underline(),
            ProfileSource::Embedded(_) => "Embedded".blue(),
        };

        match item.profile.get()? {
            Some(profile) => profile,
            None => bail!(
                "The profile '{}' at {path} no longer exists.",
                item.config.name,
            ),
        }
    } else {
        get_active_profile(config)?.1.to_ref()
    };

    write_profile(output_path, &profile)?;

    Ok(())
}
