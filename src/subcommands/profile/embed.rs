use std::fs;

use anyhow::{bail, Result};
use libium::config::{
    read_profile,
    structs::{Config, ProfileSource},
};

use crate::get_active_profile_index;

pub async fn embed(config: &mut Config, name: Option<String>) -> Result<()> {
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

    let path = match &config.profiles[index].profile {
        ProfileSource::Path(path) => path,
        ProfileSource::Embedded(_) => bail!("The profile is already embedded"),
    }
    .clone();

    let Some(profile) = read_profile(&path)? else {
        bail!("The profile at '{}' no longer exists.", path.display())
    };

    config.profiles[index].profile = ProfileSource::Embedded(Box::new(profile));
    fs::remove_file(path)?;

    Ok(())
}
