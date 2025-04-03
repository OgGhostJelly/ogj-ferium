use colored::Colorize;
use libium::{
    config::structs::{Profile, ProfileItem},
    iter_ext::IterExt as _,
};

pub fn info(profile_item: &ProfileItem, profile: &Profile, active: bool) {
    let name = profile_item.name.bold();
    let is_active = if active { " *" } else { "" };

    let profile_path = profile_item.path.display().to_string().blue().underline();

    let mods_dir = profile_item
        .mods_dir
        .display()
        .to_string()
        .blue()
        .underline();

    let shaderpacks_dir = profile_item
        .shaderpacks_dir
        .display()
        .to_string()
        .blue()
        .underline();

    let resourcepacks_dir = profile_item
        .resourcepacks_dir
        .display()
        .to_string()
        .blue()
        .underline();

    let version = if let Some(versions) = &profile.filters.versions {
        versions.iter().map(|v| v.to_string().green()).display(", ")
    } else {
        "Any".into()
    };

    let mod_loader = profile
        .filters
        .mod_loaders
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .map(|l| l.to_string().purple())
        .display(" or ");

    let mods = profile.mods.len().to_string().yellow();

    println!(
        "{name}{is_active}
        \r  Profile Path:       {profile_path}
        \r  Mods directory:     {mods_dir}
        \r  Respacks directory: {resourcepacks_dir}
        \r  Shaders directory:  {shaderpacks_dir}
        \r  Minecraft Version:  {version}
        \r  Mod Loader:         {mod_loader}
        \r  Mods:               {mods}\n"
    );
}
