mod legacy;
pub mod modpack;
pub mod options;
pub mod structs;
use std::{
    collections::HashMap,
    fs::{self, create_dir_all},
    io::Result,
    path::Path,
};

use options::OptionsOverrides;
use thiserror::Error;

/// Open the config file at `path` and deserialise it into a config struct
pub fn read_config(path: impl AsRef<Path>) -> Result<structs::Config> {
    if !path.as_ref().exists() {
        create_dir_all(path.as_ref().parent().expect("Invalid config directory"))?;
        write_config(&path, &structs::Config::default())?;
    }

    let contents = fs::read_to_string(&path)?;
    let config: structs::Config = toml::from_str(&contents).map_err(invalid_data_to_io)?;

    Ok(config)
}

fn invalid_data_to_io<E>(error: E) -> std::io::Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    std::io::Error::new(std::io::ErrorKind::InvalidData, error)
}

pub fn read_profile(path: impl AsRef<Path>) -> Result<Option<structs::Profile>> {
    let file = match fs::read_to_string(path) {
        Ok(file) => file,
        Err(e) if matches!(e.kind(), std::io::ErrorKind::NotFound) => return Ok(None),
        Err(e) => return Err(e),
    };

    let profile: structs::Profile = toml::from_str(&file).map_err(invalid_data_to_io)?;

    Ok(Some(profile))
}

/// Serialise `config` and write it to the config file at `path`
pub fn write_config(path: impl AsRef<Path>, config: &structs::Config) -> Result<()> {
    let contents = toml::to_string(config).map_err(invalid_data_to_io)?;
    fs::write(path, contents)?;
    Ok(())
}

/// Serialise `profile` and write it to the profile file at `path`
pub fn write_profile(path: impl AsRef<Path>, profile: &structs::Profile) -> Result<()> {
    let contents = toml::to_string(profile).map_err(invalid_data_to_io)?;
    fs::write(path, contents)?;
    Ok(())
}

pub fn migrate_legacy_config(
    old_config_path: impl AsRef<Path>,
) -> std::result::Result<(), MigrateError> {
    let empty: &Path = Path::new("");
    let config = legacy::read_config(old_config_path.as_ref())?;

    let mut profiles = vec![];

    for legacy_profile in config.profiles {
        let legacy::structs::Profile {
            name,
            output_dir: mods_dir,
            filters,
            mods: legacy_mods,
            ..
        } = legacy_profile;

        let profile = structs::Profile {
            filters: legacy::migrate_filters(filters)?,
            imports: Vec::new(),
            options: OptionsOverrides::default(),
            overrides: None,
            mods: {
                let mut mods = HashMap::new();
                for mod_ in legacy_mods {
                    let id = mod_.slug.clone().unwrap_or(mod_.name.clone());
                    mods.insert(id, mod_.try_into()?);
                }
                mods
            },
            shaders: HashMap::new(),
            modpacks: HashMap::new(),
            resourcepacks: HashMap::new(),
        };

        let minecraft_dir = mods_dir.parent().unwrap_or(empty).to_path_buf();

        let item = structs::ProfileItem::new(
            structs::ProfileSource::Embedded(Box::new(profile)),
            name,
            minecraft_dir,
        );

        profiles.push(item);
    }

    for legacy_modpack in config.modpacks {
        let minecraft_dir = legacy_modpack
            .output_dir
            .parent()
            .unwrap_or(empty)
            .to_path_buf();
        let source: structs::Source = legacy_modpack.identifier.into();

        let profile = structs::Profile {
            filters: structs::Filters::empty(),
            imports: Vec::new(),
            options: OptionsOverrides::default(),
            overrides: None,
            mods: HashMap::new(),
            shaders: HashMap::new(),
            modpacks: HashMap::from([(legacy_modpack.name.clone(), source)]),
            resourcepacks: HashMap::new(),
        };

        profiles.push(structs::ProfileItem {
            profile: structs::ProfileSource::Embedded(Box::new(profile)),
            config: structs::ProfileItemConfig {
                name: legacy_modpack.name,
                minecraft_dir,
            },
        })
    }

    let config = structs::Config {
        active_profile: config.active_profile,
        profiles,
    };

    let mut out_config = old_config_path.as_ref().to_path_buf();
    out_config.set_file_name(match out_config.file_name().and_then(|o| o.to_str()) {
        Some(file_name) => format!("ogj-{file_name}"),
        None => "ogj-config".to_owned(),
    });
    out_config.set_extension("toml");

    write_config(out_config, &config)?;

    Ok(())
}

#[derive(Error, Debug)]
#[error(transparent)]
pub enum MigrateError {
    Semver(#[from] semver::Error),
    Regex(#[from] regex::Error),
    IO(#[from] std::io::Error),
}
