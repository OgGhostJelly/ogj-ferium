use std::path::PathBuf;

use anyhow::{bail, Result};
use libium::config::{
    structs::{Config, ProfileSource},
    write_profile,
};

use crate::get_active_profile_index;

pub async fn unembed(
    config: &mut Config,
    output_path: Option<PathBuf>,
    name: Option<String>,
) -> Result<()> {
    let index = if let Some(name) = name {
        let Some(item) = config
            .profiles
            .iter()
            .enumerate()
            .find(|(_, item)| item.config.name.eq_ignore_ascii_case(&name))
        else {
            bail!("A profile with that name doesnt exist");
        };

        item.0
    } else {
        get_active_profile_index(config)?
    };

    let output_path = match output_path {
        Some(output_path) => output_path,
        None => (config.profiles[index].config.name.clone() + ".toml").into(),
    };

    match &config.profiles[index].profile {
        ProfileSource::Path(_) => bail!("The profile is already unembedded"),
        ProfileSource::Embedded(profile) => write_profile(&output_path, profile)?,
    }

    config.profiles[index].profile = ProfileSource::Path(output_path);

    Ok(())
}
