use colored::Colorize;
use libium::{
    config::structs::{ProfileItemConfig, ProfileSourceMut},
    iter_ext::IterExt as _,
};

pub fn info(profile_item: &ProfileItemConfig, profile: &ProfileSourceMut, active: bool) {
    let name = if active {
        profile_item.name.bold().italic()
    } else {
        profile_item.name.bold()
    };
    let is_active = if active { " *" } else { "" };

    let profile_path = match profile {
        ProfileSourceMut::Path(path, _) => path.display().to_string().blue().underline(),
        ProfileSourceMut::Embedded(_) => "Embedded".blue(),
    };

    let minecraft_dir = profile_item
        .minecraft_dir
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

    let sources = (profile.mods.len()
        + profile.resourcepacks.len()
        + profile.shaders.len()
        + profile.modpacks.len())
    .to_string()
    .yellow();

    println!(
        "{name}{is_active}
        \r  Profile Path:       {profile_path}
        \r  Minecraft Dir:      {minecraft_dir}
        \r  Minecraft Version:  {version}
        \r  Mod Loader:         {mod_loader}
        \r  Sources:            {sources}\n"
    );
}
