use anyhow::{bail, Result};
use colored::Colorize as _;
use inquire::MultiSelect;
use libium::{
    config::structs::{Profile, SourceId},
    iter_ext::IterExt as _,
};

/// If `to_remove` is empty, display a list of projects in the profile to select from and remove selected ones
///
/// Else, search the given strings with the projects' name and IDs and remove them
pub fn remove(profile: &mut Profile, to_remove: Vec<String>) -> Result<()> {
    let keys_to_remove = if to_remove.is_empty() {
        let mod_ids = profile.mods.iter().map(|(name, _)| name).collect_vec();
        let mod_info = mod_ids
            .iter()
            .map(|name| format!("{name:11}"))
            .collect_vec();
        MultiSelect::new("Select mods to remove", mod_info)
            .raw_prompt_skippable()?
            .unwrap_or_default()
            .iter()
            .map(|o| o.index)
            .map(|i| mod_ids[i].to_owned())
            .collect_vec()
    } else {
        let mut items_to_remove = Vec::new();

        for to_remove in to_remove {
            if let Some((name, _)) = profile.mods.iter().find(|(name, source)| {
                name.eq_ignore_ascii_case(&to_remove)
                    || source.ids().any(|id| match id {
                        SourceId::Curseforge(id) => id.to_string() == to_remove,
                        SourceId::Modrinth(id) => *id == to_remove,
                        SourceId::Github(owner, repo) => {
                            format!("{owner}/{repo}").eq_ignore_ascii_case(&to_remove)
                        }
                        _ => todo!(),
                    })
            }) {
                items_to_remove.push(name.to_owned());
            } else {
                bail!("A mod with ID or name {to_remove} is not present in this profile");
            }
        }

        items_to_remove
    };

    let mut removed = Vec::new();
    for key in keys_to_remove {
        profile.mods.remove(&key);
        removed.push(key);
    }

    if !removed.is_empty() {
        println!(
            "Removed {}",
            removed.iter().map(|txt| txt.bold()).display(", ")
        );
    }

    Ok(())
}
