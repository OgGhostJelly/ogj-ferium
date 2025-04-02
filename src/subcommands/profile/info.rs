use colored::Colorize;
use libium::{
    config::structs::{Profile, ProfileItem},
    iter_ext::IterExt as _,
};

pub fn info(profile_item: &ProfileItem, profile: &Profile, active: bool) {
    println!(
        "{}{}
        \r  Profile Path:       {}
        \r  Output directory:   {}{}{}
        \r  Mods:               {}\n",
        profile_item.name.bold(),
        if active { " *" } else { "" },
        profile_item.path.display().to_string().blue().underline(),
        profile_item
            .mods_dir
            .display()
            .to_string()
            .blue()
            .underline(),
        if let Some(versions) = &profile.filters.versions {
            format!(
                "\n  Minecraft Version:  {}",
                versions.iter().map(|v| v.to_string().green()).display(", ")
            )
        } else {
            format!("\n  Minecraft Version:  Any")
        },
        format!(
            "\n  Mod Loader:         {}",
            profile
                .filters
                .mod_loaders
                .as_ref()
                .unwrap_or(&vec![])
                .iter()
                .map(|l| l.to_string().purple())
                .display(" or ")
        ),
        profile.mods.len().to_string().yellow(),
    );
}
