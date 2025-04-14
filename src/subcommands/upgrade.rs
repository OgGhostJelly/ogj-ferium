use crate::{
    default_semaphore,
    download::{clean, download, read_overrides, InstallData},
    CROSS, SEMAPHORE, STYLE_NO, TICK,
};
use anyhow::{anyhow, bail, Context as _, Result};
use colored::Colorize as _;
use indicatif::ProgressBar;
use libium::{
    config::{
        modpack::{curseforge, modrinth, read_file_from_zip, zip_extract},
        structs::{
            Filters, ModLoader, Profile, ProfileItemConfig, Source, SourceId, SourceKind,
            SourceKindWithModpack, Version,
        },
    },
    upgrade::{
        from_modpack_file, mod_downloadable, try_from_cf_file, DistributionDeniedError,
        DownloadData,
    },
    CURSEFORGE_API, HOME,
};
use parking_lot::Mutex;
use std::{
    fs::{self, File},
    io::BufReader,
    mem::take,
    path::{Path, PathBuf},
    sync::{mpsc, Arc, LazyLock},
    time::Duration,
};
use tokio::task::JoinSet;

pub async fn upgrade(
    profile_item: &ProfileItemConfig,
    profile: &Profile,
    filters: Filters,
) -> Result<()> {
    let filters = filters.concat(profile.filters.clone());
    check_unstrict_filter(&filters);
    println!("{}", "Upgrading Sources".bold());

    let mut to_download = vec![];
    let mut to_install = vec![];

    let error =
        get_platform_downloadables(&mut to_download, &mut to_install, profile, &filters).await?;

    for kind in SourceKind::ARRAY {
        clean(
            &profile_item.minecraft_dir.join(kind.directory()),
            &mut to_download,
            &mut to_install,
            matches!(kind, SourceKind::Mods),
        )
        .await?;
    }

    if to_download.is_empty() && to_install.is_empty() {
        println!("\n{}", "All up to date!".bold());
    } else {
        println!("{}", "\nDownloading Source Files\n".bold());
        download(profile_item.minecraft_dir.clone(), to_download, to_install).await?;
    }

    if error {
        Err(anyhow!(
            "\nCould not get the latest compatible version of some sources"
        ))
    } else {
        Ok(())
    }
}

/// Get the latest compatible downloadable for the sources in `profile`
///
/// If an error occurs with a resolving task, instead of failing immediately,
/// resolution will continue and the error return flag is set to true.
async fn get_platform_downloadables(
    to_download: &mut Vec<DownloadData>,
    to_install: &mut Vec<InstallData>,
    profile: &Profile,
    filters: &Filters,
) -> Result<bool> {
    let mut error = false;

    for kind in SourceKind::ARRAY {
        if profile.map(*kind).is_empty() {
            continue;
        }

        let new_error =
            get_source_downloadables(*kind, to_download, to_install, profile, filters).await?;

        error = error || new_error;
    }

    Ok(error)
}

async fn get_source_downloadables(
    kind: SourceKind,
    to_download: &mut Vec<DownloadData>,
    to_install: &mut Vec<InstallData>,
    profile: &Profile,
    filters: &Filters,
) -> Result<bool> {
    let progress_bar = Arc::new(Mutex::new(ProgressBar::new(0).with_style(STYLE_NO.clone())));
    let mut tasks = JoinSet::new();
    let mut done_sources = Vec::new();
    let client = reqwest::Client::new();
    let (mod_sender, mod_rcvr) = mpsc::channel();

    // Wrap it again in an Arc so that I can count the references to it,
    // because I cannot drop the main thread's sender due to the recursion
    let mod_sender = Arc::new(mod_sender);

    println!("{}\n", "Determining the Latest Compatible Versions".bold());
    progress_bar
        .lock()
        .enable_steady_tick(Duration::from_millis(100));
    let sources = profile.map(kind);
    let pad_len = sources
        .iter()
        .map(|(name, _)| name.len())
        .max()
        .unwrap_or(20)
        .clamp(20, 50);

    for (name, source) in sources {
        mod_sender.send((name.to_owned(), source.clone()))?;
    }

    let mut initial = true;

    // A race condition exists where if the last task drops its sender before this thread receives the message,
    // that particular message will get ignored. I used the ostrich algorithm to solve this.

    // `initial` accounts for the edge case where at first,
    // no tasks have been spawned yet but there are messages in the channel
    while Arc::strong_count(&mod_sender) > 1 || initial {
        if let Ok((name, source)) = mod_rcvr.try_recv() {
            initial = false;

            if done_sources.contains(&name) {
                continue;
            }

            done_sources.push(name.clone());
            progress_bar.lock().inc_length(1);

            let filters = filters.clone();
            let dep_sender = Arc::clone(&mod_sender);
            let progress_bar = Arc::clone(&progress_bar);
            let client = client.clone();

            tasks.spawn(async move {
                let permit = SEMAPHORE.get_or_init(default_semaphore).acquire().await?;

                let result = source.fetch_download_file(kind, vec![&filters]).await;

                drop(permit);

                progress_bar.lock().inc(1);
                match result {
                    Ok(mut download_file) => {
                        progress_bar.lock().println(format!(
                            "{} {name:pad_len$}  {}",
                            TICK.clone(),
                            download_file.filename().dimmed()
                        ));
                        for dep in take(&mut download_file.dependencies) {
                            let id = format!(
                                "Dependency of {name}: {}",
                                match &dep {
                                    SourceId::Curseforge(id) => id.to_string(),
                                    SourceId::Modrinth(id) | SourceId::PinnedModrinth(id, _) =>
                                        id.to_owned(),
                                    _ => unreachable!(),
                                }
                            );
                            let source = Source::from_id(dep, Filters::empty());
                            dep_sender.send((id, source))?;
                        }
                        if let SourceKind::Modpacks = kind {
                            let install_overrides = source
                                .filters()
                                .and_then(|filters| filters.install_overrides)
                                .unwrap_or(true);

                            let mut to_download = vec![];
                            let mut to_install = vec![];
                            download_modpack(
                                &mut to_download,
                                &mut to_install,
                                client,
                                download_file,
                                install_overrides,
                            )
                            .await?;
                            Ok(Some((to_download, to_install)))
                        } else {
                            Ok(Some((vec![download_file], vec![])))
                        }
                    }
                    Err(err) => {
                        if let mod_downloadable::Error::ModrinthError(
                            ferinth::Error::RateLimitExceeded(_),
                        ) = err
                        {
                            // Immediately fail if the rate limit has been exceeded
                            progress_bar.lock().finish_and_clear();
                            bail!(err);
                        }
                        progress_bar.lock().println(format!(
                            "{}",
                            format!("{CROSS} {name:pad_len$}  {err}").red()
                        ));
                        Ok(None)
                    }
                }
            });
        }
    }

    Arc::try_unwrap(progress_bar)
        .map_err(|_| anyhow!("Failed to run threads to completion"))?
        .into_inner()
        .finish_and_clear();

    let tasks = tasks
        .join_all()
        .await
        .into_iter()
        .collect::<Result<Vec<_>>>()?;

    let error = tasks.iter().any(Option::is_none);
    for (new_to_download, new_to_install) in tasks.into_iter().flatten() {
        for downloadable in new_to_download {
            to_download.push(downloadable);
        }
        for installable in new_to_install {
            to_install.push(installable);
        }
    }

    Ok(error)
}

pub static TMP_DIR: LazyLock<PathBuf> =
    LazyLock::new(|| HOME.join(".config").join("ferium").join(".tmp"));

async fn download_modpack(
    to_download: &mut Vec<DownloadData>,
    to_install: &mut Vec<InstallData>,
    client: reqwest::Client,
    downloadable: DownloadData,
    install_overrides: bool,
) -> Result<()> {
    let (_size, filename) = downloadable
        .download(client, TMP_DIR.as_path(), |_| {})
        .await?;
    let path = TMP_DIR.join(filename);
    let res = download_modpack_inner(to_download, to_install, &path, install_overrides).await;
    fs::remove_file(path)?;
    res
}

async fn download_modpack_inner(
    to_download: &mut Vec<DownloadData>,
    to_install: &mut Vec<InstallData>,
    path: &PathBuf,
    install_overrides: bool,
) -> Result<()> {
    let Some(kind) = SourceKindWithModpack::infer(path)? else {
        bail!("Couldn't infer the modpack type")
    };

    let modpack_file = File::open(path)?;

    match kind {
        SourceKindWithModpack::ModpacksCurseforge => {
            let manifest: curseforge::Manifest = serde_json::from_str(
                &read_file_from_zip(BufReader::new(modpack_file), "manifest.json")?
                    .context("Does not contain manifest")?,
            )?;

            let file_ids = manifest.files.iter().map(|file| file.file_id).collect();
            let files = CURSEFORGE_API.get_files(file_ids).await?;

            let mut tasks = JoinSet::new();
            let mut msg_shown = false;
            for file in files {
                match try_from_cf_file(SourceKind::Modpacks, file, None) {
                    Ok((_metadata, mut downloadable)) => {
                        downloadable.output = PathBuf::from(
                            if Path::new(&downloadable.filename())
                                .extension()
                                .is_some_and(|ext| ext.eq_ignore_ascii_case(".zip"))
                            {
                                "resourcepacks"
                            } else {
                                "mods"
                            },
                        )
                        .join(downloadable.filename());
                        to_download.push(downloadable);
                    }
                    Err(DistributionDeniedError(mod_id, file_id)) => {
                        if !msg_shown {
                            println!("\n{}", "The following mod(s) have denied 3rd parties such as Ferium from downloading it".red().bold());
                        }
                        msg_shown = true;
                        tasks.spawn(async move {
                            let project = CURSEFORGE_API.get_mod(mod_id).await?;
                            eprintln!(
                                "- {}
                           \r  {}",
                                project.name.bold(),
                                format!("{}/download/{file_id}", project.links.website_url)
                                    .blue()
                                    .underline(),
                            );
                            Ok::<(), furse::Error>(())
                        });
                    }
                }
            }

            if install_overrides {
                let tmp_dir = TMP_DIR.join(manifest.name);
                zip_extract(path, &tmp_dir)?;
                read_overrides(to_install, &tmp_dir.join(manifest.overrides))?;
            }
        }
        SourceKindWithModpack::ModpacksModrinth => {
            let metadata: modrinth::Metadata = serde_json::from_str(
                &read_file_from_zip(BufReader::new(modpack_file), "modrinth.index.json")?
                    .context("Does not contain metadata file")?,
            )?;

            for file in metadata.files {
                to_download.push(from_modpack_file(file));
            }

            if install_overrides {
                let tmp_dir = TMP_DIR.join(metadata.name);
                zip_extract(path, &tmp_dir)?;
                read_overrides(to_install, &tmp_dir.join("overrides"))?;
            }
        }
        _ => bail!("That is not a modpack!"),
    }

    Ok(())
}

/// Warn if a filter is potentially not strict enough.
fn check_unstrict_filter(filters: &Filters) {
    if let Some(mod_loaders) = &filters.mod_loaders {
        check_unstrict_mod_loaders(mod_loaders);
    }

    if let Some(versions) = &filters.versions {
        check_unstrict_versions(versions);
    }
}

fn check_unstrict_versions(versions: &Vec<Version>) {
    for version in versions {
        let version = version.clone().into_req();
        if version
            .comparators
            .iter()
            .any(|comp| comp.minor.is_some() && comp.patch.is_some())
        {
            return;
        }
    }

    println!(
        "{}",
        "Warning: potentially lax version requirements"
            .yellow()
            .bold()
    );
}

fn check_unstrict_mod_loaders(mod_loaders: &Vec<ModLoader>) {
    let mut is_err = false;
    let mut loader = None;

    for mod_loader in mod_loaders {
        match mod_loader {
            ModLoader::Fabric | ModLoader::Quilt => {
                if loader.is_some() && loader != Some(ModLoader::Fabric) {
                    is_err = true;
                    break;
                }

                loader = Some(ModLoader::Fabric);
            }
            ModLoader::Forge => {
                if loader.is_some() && loader != Some(ModLoader::Forge) {
                    is_err = true;
                    break;
                }

                loader = Some(ModLoader::Fabric);
            }
            ModLoader::NeoForge => {
                if loader.is_some() && loader != Some(ModLoader::NeoForge) {
                    is_err = true;
                    break;
                }

                loader = Some(ModLoader::NeoForge);
            }
        }
    }

    if is_err {
        println!(
            "{}",
            "Warning: specified multiple possible mod loaders"
                .yellow()
                .bold()
        );
    }
}
