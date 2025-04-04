use std::path::PathBuf;

use anyhow::{bail, Result};
use colored::Colorize;
use libium::config::{
    read_profile,
    structs::{Config, ProfileSource},
    write_profile,
};

use crate::get_active_profile;

pub async fn export(config: &mut Config, output_path: PathBuf, name: Option<String>) -> Result<()> {
    let profile = if let Some(name) = name {
        let Some(item) = config
            .profiles
            .iter()
            .find(|item| item.name.eq_ignore_ascii_case(&name))
        else {
            bail!("A profile with that name doesnt exist");
        };

        match &item.profile {
            ProfileSource::Path(path) => match read_profile(path)? {
                Some(profile) => profile,
                None => bail!(
                    "The profile '{}' at {} no longer exists.",
                    item.name,
                    path.display().to_string().blue().underline(),
                ),
            },
            ProfileSource::Embedded(profile) => *profile.clone(),
        }
    } else {
        get_active_profile(config)?.1
    };

    write_profile(output_path, &profile)?;

    Ok(())
}
