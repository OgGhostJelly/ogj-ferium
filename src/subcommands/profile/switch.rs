use anyhow::{anyhow, Result};
use colored::Colorize as _;
use inquire::Select;
use libium::{config::structs::Config, iter_ext::IterExt as _};

use crate::try_iter_profiles;

#[derive(clap::Args, Clone, Debug)]
/// Switch between different profiles.
/// Optionally, provide the name of the profile to switch to.
pub struct Args {
    /// The name of the profile to switch to
    pub profile_name: Option<String>,
}

pub fn switch(config: &mut Config, Args { profile_name }: Args) -> Result<()> {
    if config.profiles.len() <= 1 {
        Err(anyhow!("There is only 1 profile in your config"))
    } else if let Some(profile_name) = profile_name {
        match config
            .profiles
            .iter()
            .position(|item| item.config.name.eq_ignore_ascii_case(&profile_name))
        {
            Some(selection) => {
                config.active_profile = selection;
                Ok(())
            }
            None => Err(anyhow!("The profile provided does not exist")),
        }
    } else {
        let profile_info = try_iter_profiles(&mut config.profiles)
            .map(|(item, profile)| {
                format!(
                    "{:8} {:7} {} {}",
                    profile
                        .filters
                        .mod_loaders
                        .as_ref()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|l| l.to_string().purple())
                        .display(" or "),
                    profile
                        .filters
                        .versions
                        .as_ref()
                        .unwrap_or(&vec![])
                        .iter()
                        .map(|v| v.to_string().green())
                        .display(", "),
                    item.name.bold(),
                    format!("({} mods)", profile.mods.len()).yellow(),
                )
            })
            .collect_vec();

        let mut select = Select::new("Select which profile to switch to", profile_info);
        if config.active_profile < config.profiles.len() {
            select.starting_cursor = config.active_profile;
        }
        if let Ok(selection) = select.raw_prompt() {
            config.active_profile = selection.index;
        }
        Ok(())
    }
}
