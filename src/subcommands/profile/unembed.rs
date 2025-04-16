use std::path::PathBuf;

use anyhow::{bail, Result};
use libium::config::{
    structs::{Config, ProfileSource},
    write_profile,
};

use crate::get_active_profile_index;

/// Unembed a profile
#[derive(clap::Args, Clone, Debug)]
pub struct Args {
    /// Where to output the profile or the profile name suffixed with `.toml` by default
    #[clap(long, short)]
    #[clap(value_hint(clap::ValueHint::FilePath))]
    pub output_path: Option<PathBuf>,
    /// The name of the profile or the active profile by default
    #[clap(long, short)]
    pub name: Option<String>,
}

pub async fn unembed(config: &mut Config, Args { output_path, name }: Args) -> Result<()> {
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
