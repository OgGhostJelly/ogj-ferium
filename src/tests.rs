#![expect(clippy::unwrap_used)]

use crate::{
    actual_main,
    cli::{Ferium, FilterArguments, ModpackSubCommands, Platform, ProfileSubCommands, SubCommands},
};
use libium::config::structs::{self, ModLoader};
use std::{
    env::current_dir, fmt, fs::{copy, create_dir_all, File}, path::PathBuf
};

macro_rules! assert_matches {
    ($left:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )? $(,)?) => {
        match $left {
            $( $pattern )|+ $( if $guard )? => {}
            ref left_val => {
                assert_matches_failed(
                    left_val,
                    std::stringify!($($pattern)|+ $(if $guard)?),
                    std::option::Option::None
                );
            }
        }
    };
    ($left:expr, $(|)? $( $pattern:pat_param )|+ $( if $guard: expr )?, $($arg:tt)+) => {
        match $left {
            $( $pattern )|+ $( if $guard )? => {}
            ref left_val => {
                assert_matches_failed(
                    left_val,
                    std::stringify!($($pattern)|+ $(if $guard)?),
                    std::option::Option::Some($crate::format_args!($($arg)+))
                );
            }
        }
    };
}

const DEFAULT: Ferium = Ferium {
    subcommand: SubCommands::Profile { subcommand: None },
    threads: None,
    parallel_network: 10,
    github_token: None,
    curseforge_api_key: None,
    config_file: None,
    no_gui: Some(true),
};

fn get_args(subcommand: SubCommands, config_file: Option<&str>) -> Ferium {
    let running = PathBuf::from(".")
        .join("tests")
        .join("configs")
        .join("running")
        .join(format!("{:X}.json", rand::random::<usize>()));
    let _ = create_dir_all(running.parent().unwrap());
    if let Some(config_file) = config_file {
        let path = format!("./tests/configs/{config_file}.json");
        let mut config: structs::Config = serde_json::from_reader(File::open(path).unwrap()).unwrap();

        for item in &mut config.profiles {
            let running = format!("./tests/profiles/running/{:X}.json", rand::random::<usize>()).into();
            copy(&item.path, &running).unwrap();
            item.path = running;
        }

        serde_json::to_writer(File::create(&running).unwrap(), &config).unwrap();
    }
    Ferium {
        subcommand,
        config_file: Some(running.into()),
        ..DEFAULT
    }
}

// TODO
// #[tokio::test(flavor = "multi_thread")]
// async fn arg_parse() {}

#[tokio::test(flavor = "multi_thread")]
async fn create_profile_no_profiles_to_import() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Create {
                    // There should be no other profiles to import mods from
                    import: Some(None),
                    game_version: vec!["1.21.4".to_owned()],
                    mod_loader: Some(ModLoader::Fabric),
                    name: Some("Test Profile".to_owned()),
                    output_dir: Some(current_dir().unwrap().join("tests").join("mods")),
                    path: Some(current_dir().unwrap().join("tests/profiles/running")
                        .join(format!("{:X}.json", rand::random::<usize>())))
                })
            },
            None,
        ))
        .await,
        Err(_),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_profile_rel_dir() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Create {
                    // There should be no other profiles to import mods from
                    import: Some(None),
                    game_version: vec!["1.21.4".to_owned()],
                    mod_loader: Some(ModLoader::Fabric),
                    name: Some("Test Profile".to_owned()),
                    output_dir: Some(PathBuf::from(".").join("tests").join("mods")),
                    path: Some(current_dir().unwrap().join("tests/profiles/running")
                        .join(format!("{:X}.json", rand::random::<usize>())))
                })
            },
            None,
        ))
        .await,
        Err(_),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_profile_import_mods() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Create {
                    // There should be no other profiles to import mods from
                    import: Some(Some("Default Modded".to_owned())),
                    game_version: vec!["1.21.4".to_owned()],
                    mod_loader: Some(ModLoader::Fabric),
                    name: Some("Test Profile".to_owned()),
                    output_dir: Some(current_dir().unwrap().join("tests").join("mods")),
                    path: Some(current_dir().unwrap().join("tests/profiles/running")
                        .join(format!("{:X}.json", rand::random::<usize>())))
                })
            },
            Some("one_profile_full"),
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_profile_existing_name() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Create {
                    import: None,
                    game_version: vec!["1.21.4".to_owned()],
                    mod_loader: Some(ModLoader::Fabric),
                    name: Some("Default Modded".to_owned()),
                    output_dir: Some(current_dir().unwrap().join("tests").join("mods")),
                    path: Some(current_dir().unwrap().join("tests/profiles/running")
                        .join(format!("{:X}.json", rand::random::<usize>())))
                })
            },
            None,
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn create_profile() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Create {
                    import: None,
                    game_version: vec!["1.21.4".to_owned()],
                    mod_loader: Some(ModLoader::Fabric),
                    name: Some("Test Profile".to_owned()),
                    output_dir: Some(current_dir().unwrap().join("tests").join("mods")),
                    path: Some(current_dir().unwrap().join("tests/profiles/running")
                        .join(format!("{:X}.json", rand::random::<usize>())))
                })
            },
            None,
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn add_modrinth() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Add {
                identifiers: vec!["starlight".to_owned()],
                force: false,
                filters: FilterArguments::default(),
            },
            Some("empty_profile"),
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn add_curseforge() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Add {
                identifiers: vec!["591388".to_owned()],
                force: false,
                filters: FilterArguments::default(),
            },
            Some("empty_profile"),
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn add_github() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Add {
                identifiers: vec!["CaffeineMC/sodium".to_owned()],
                force: false,
                filters: FilterArguments::default(),
            },
            Some("empty_profile"),
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn add_all() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Add {
                identifiers: vec![
                    "starlight".to_owned(),
                    "591388".to_owned(),
                    "CaffeineMC/sodium".to_owned()
                ],
                force: false,
                filters: FilterArguments::default(),
            },
            Some("empty_profile"),
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn already_added() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Add {
                identifiers: vec![
                    "starlight".to_owned(),
                    "591388".to_owned(),
                    "CaffeineMC/sodium".to_owned()
                ],
                force: false,
                filters: FilterArguments::default(),
            },
            Some("one_profile_full"),
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn scan() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Scan {
                platform: Platform::default(),
                directory: Some(current_dir().unwrap().join("tests").join("test_mods")),
                force: false,
            },
            Some("empty_profile"),
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn modpack_add_modrinth() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Modpack {
                subcommand: Some(ModpackSubCommands::Add {
                    identifier: "1KVo5zza".to_owned(),
                    output_dir: Some(current_dir().unwrap().join("tests").join("mods")),
                    install_overrides: Some(true),
                })
            },
            Some("empty")
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn modpack_add_curseforge() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Modpack {
                subcommand: Some(ModpackSubCommands::Add {
                    identifier: "452013".to_owned(),
                    output_dir: Some(current_dir().unwrap().join("tests").join("mods")),
                    install_overrides: Some(true),
                })
            },
            Some("empty")
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn list_no_profile() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::List {
                verbose: false,
                markdown: false
            },
            Some("empty"),
        ))
        .await,
        Err(_),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn list_empty_profile() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::List {
                verbose: false,
                markdown: false
            },
            Some("empty_profile"),
        ))
        .await,
        Err(_),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn list() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::List {
                verbose: false,
                markdown: false
            },
            Some("one_profile_full"),
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn list_verbose() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::List {
                verbose: true,
                markdown: false
            },
            Some("one_profile_full"),
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn list_markdown() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::List {
                verbose: true,
                markdown: true
            },
            Some("one_profile_full"),
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn list_profiles() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Profiles,
            Some("two_profiles_one_empty"),
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn list_modpacks() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Modpacks,
            Some("two_modpacks_mdactive"),
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn upgrade() {
    assert_matches!(
        actual_main(get_args(SubCommands::Upgrade, Some("one_profile_full"))).await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn upgrade_md_modpacks() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Modpack {
                subcommand: Some(ModpackSubCommands::Upgrade)
            },
            Some("two_modpacks_mdactive")
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn upgrade_cf_modpack() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Modpack {
                subcommand: Some(ModpackSubCommands::Upgrade)
            },
            Some("two_modpacks_cfactive")
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn profile_switch() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Switch {
                    profile_name: Some("Profile Two".to_owned())
                })
            },
            Some("two_profiles_one_empty")
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn modpack_switch() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Modpack {
                subcommand: Some(ModpackSubCommands::Switch {
                    modpack_name: Some("MR Fabulously Optimised".to_owned())
                })
            },
            Some("two_modpacks_cfactive")
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_fail() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Remove {
                mod_names: vec![
                    "starlght (fabric)".to_owned(),
                    "incendum".to_owned(),
                    "sodum".to_owned(),
                ]
            },
            Some("two_profiles_one_empty")
        ))
        .await,
        Err(_),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_name() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Remove {
                mod_names: vec![
                    "starlight (fabric)".to_owned(),
                    "incendium".to_owned(),
                    "sodium".to_owned(),
                ]
            },
            Some("two_profiles_one_empty")
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn remove_id() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Remove {
                mod_names: vec![
                    "H8CaAYZC".to_owned(),
                    "591388".to_owned(),
                    "caffeinemc/sodium".to_owned(),
                ]
            },
            Some("two_profiles_one_empty")
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_profile() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Profile {
                subcommand: Some(ProfileSubCommands::Delete {
                    profile_name: Some("Profile Two".to_owned()),
                    switch_to: None
                })
            },
            Some("two_profiles_one_empty")
        ))
        .await,
        Ok(()),
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn delete_modpack() {
    assert_matches!(
        actual_main(get_args(
            SubCommands::Modpack {
                subcommand: Some(ModpackSubCommands::Delete {
                    modpack_name: Some("MR Fabulously Optimised".to_owned()),
                    switch_to: None
                })
            },
            Some("two_modpacks_cfactive")
        ))
        .await,
        Ok(()),
    );
}

fn assert_matches_failed<T: fmt::Debug + ?Sized>(
    left: &T,
    right: &str,
    args: Option<fmt::Arguments<'_>>,
) -> ! {
    // The pattern is a string so it can be displayed directly.
    struct Pattern<'a>(&'a str);
    impl fmt::Debug for Pattern<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(self.0)
        }
    }

    let right = &Pattern(right);

    match args {
        Some(args) => panic!(
            r#"assertion `left matches right` failed: {args}
  left: {left:?}
 right: {right:?}"#
        ),
        None => panic!(
            r#"assertion `left matches right` failed
  left: {left:?}
 right: {right:?}"#
        ),
    }
}