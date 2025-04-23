use crate::config::{
    structs::{self, Filters},
    MigrateError,
};

use super::filters::Filter;
use derive_more::derive::Display;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, str::FromStr};

#[derive(Deserialize, Serialize, Debug, Default, Clone)]
pub struct Config {
    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub active_profile: usize,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub profiles: Vec<Profile>,

    #[serde(skip_serializing_if = "is_zero")]
    #[serde(default)]
    pub active_modpack: usize,

    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub modpacks: Vec<Modpack>,
}

const fn is_zero(n: &usize) -> bool {
    *n == 0
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Modpack {
    pub name: String,
    pub output_dir: PathBuf,
    pub install_overrides: bool,
    pub identifier: ModpackIdentifier,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ModpackIdentifier {
    CurseForgeModpack(i32),
    ModrinthModpack(String),
}

impl From<ModpackIdentifier> for structs::Source {
    fn from(val: ModpackIdentifier) -> Self {
        match val {
            ModpackIdentifier::CurseForgeModpack(id) => Self::curseforge(id, Filters::empty()),
            ModpackIdentifier::ModrinthModpack(id) => Self::modrinth(id, Filters::empty()),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Profile {
    pub name: String,

    /// The directory to download mod files to
    pub output_dir: PathBuf,

    // There will be no filters when reading a v4 config
    #[serde(default)]
    pub filters: Vec<Filter>,

    pub mods: Vec<Mod>,

    // Kept for backwards compatibility reasons (i.e. migrating from a v4 config)
    #[serde(skip_serializing)]
    game_version: Option<String>,
    #[serde(skip_serializing)]
    mod_loader: Option<ModLoader>,
}

impl Profile {
    /// Convert the v4 profile's `game_version` and `mod_loader` fields into filters
    pub(crate) fn backwards_compat(&mut self) {
        if let (Some(version), Some(loader)) = (self.game_version.take(), self.mod_loader.take()) {
            self.filters = vec![
                Filter::ModLoaderPrefer(match loader {
                    ModLoader::Quilt => vec![ModLoader::Quilt, ModLoader::Fabric],
                    _ => vec![loader],
                }),
                Filter::GameVersionStrict(vec![version]),
            ];
        }

        for mod_ in &self.mods {
            if mod_.check_game_version.is_some() || mod_.check_mod_loader.is_some() {
                eprintln!("WARNING: Check overrides found for {}", mod_.name);
                eprintln!("Ensure the profile migrated properly if necessary!");
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Mod {
    pub name: String,
    pub identifier: ModIdentifier,

    // Is an `Option` for backwards compatibility reasons,
    // since the slug field didn't exist in older ferium versions
    #[serde(skip_serializing_if = "Option::is_none")]
    pub slug: Option<String>,

    /// Custom filters that apply only for this mod
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub filters: Vec<Filter>,

    /// Whether the filters specified above replace or apply with the profile's filters
    #[serde(skip_serializing_if = "is_false")]
    #[serde(default)]
    pub override_filters: bool,

    // Kept for backwards compatibility reasons
    #[serde(skip_serializing)]
    check_game_version: Option<bool>,
    #[serde(skip_serializing)]
    check_mod_loader: Option<bool>,
}

impl TryInto<structs::Source> for Mod {
    type Error = MigrateError;

    fn try_into(self) -> Result<structs::Source, Self::Error> {
        let source_id = self.identifier.into();
        let filters = super::migrate_filters(self.filters)?;
        Ok(structs::Source::from_id(source_id, filters))
    }
}

const fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum ModIdentifier {
    CurseForgeProject(i32),
    ModrinthProject(String),
    GitHubRepository(String, String),

    PinnedCurseForgeProject(i32, i32),
    PinnedModrinthProject(String, String),
    PinnedGitHubRepository((String, String), i32),
}

impl From<ModIdentifier> for structs::SourceId {
    fn from(val: ModIdentifier) -> Self {
        match val {
            ModIdentifier::CurseForgeProject(id) => structs::SourceId::Curseforge(id),
            ModIdentifier::ModrinthProject(id) => structs::SourceId::Modrinth(id),
            ModIdentifier::GitHubRepository(owner, repo) => structs::SourceId::Github(owner, repo),
            ModIdentifier::PinnedCurseForgeProject(id, pin) => {
                structs::SourceId::PinnedCurseforge(id, pin)
            }
            ModIdentifier::PinnedModrinthProject(id, pin) => {
                structs::SourceId::PinnedModrinth(id, pin)
            }
            ModIdentifier::PinnedGitHubRepository(id, pin) => {
                structs::SourceId::PinnedGithub(id, pin)
            }
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Display, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ModLoader {
    Quilt,
    Fabric,
    Forge,
    #[clap(name = "neoforge")]
    NeoForge,
}

impl From<ModLoader> for structs::ModLoader {
    fn from(val: ModLoader) -> Self {
        match val {
            ModLoader::Quilt => structs::ModLoader::Quilt,
            ModLoader::Fabric => structs::ModLoader::Fabric,
            ModLoader::Forge => structs::ModLoader::Forge,
            ModLoader::NeoForge => structs::ModLoader::NeoForge,
        }
    }
}

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
#[error("The given string is not a recognised mod loader")]
pub struct ModLoaderParseError;

impl FromStr for ModLoader {
    type Err = ModLoaderParseError;

    // This implementation is case-insensitive
    fn from_str(from: &str) -> Result<Self, Self::Err> {
        match from.trim().to_lowercase().as_str() {
            "quilt" => Ok(Self::Quilt),
            "fabric" => Ok(Self::Fabric),
            "forge" => Ok(Self::Forge),
            "neoforge" => Ok(Self::NeoForge),
            _ => Err(Self::Err {}),
        }
    }
}
