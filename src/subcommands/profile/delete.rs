use crate::try_iter_profiles;

use super::switch;
use anyhow::{Context as _, Result};
use colored::Colorize as _;
use inquire::Select;
use libium::{config::structs::Config, iter_ext::IterExt as _};
use std::cmp::Ordering;

pub fn delete(
    config: &mut Config,
    profile_name: Option<String>,
    switch_to: Option<String>,
) -> Result<()> {
    // If the profile name has been provided as an option
    let selection = if let Some(profile_name) = profile_name {
        config
            .profiles
            .iter()
            .position(|item| item.config.name.eq_ignore_ascii_case(&profile_name))
            .context("The profile name provided does not exist")?
    } else {
        let profile_names = try_iter_profiles(&mut config.profiles)
            .map(|(item, profile)| {
                format!(
                    "{:6} {:7} {} {}",
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

        if let Ok(selection) = Select::new("Select which profile to delete", profile_names)
            .with_starting_cursor(config.active_profile)
            .raw_prompt()
        {
            selection.index
        } else {
            return Ok(());
        }
    };
    config.profiles.remove(selection);

    match config.active_profile.cmp(&selection) {
        // If the currently selected profile is being removed
        Ordering::Equal => {
            // And there is more than one profile
            if config.profiles.len() > 1 {
                // Let the user pick which profile to switch to
                switch(config, switch_to)?;
            } else {
                config.active_profile = 0;
            }
        }
        // If the active profile comes after the removed profile
        Ordering::Greater => {
            // Decrement the index by one
            config.active_profile -= 1;
        }
        Ordering::Less => (),
    }

    Ok(())
}
