use crate::{
    default_semaphore,
    download::{clean, download},
    CROSS, SEMAPHORE, STYLE_NO, TICK,
};
use anyhow::{anyhow, bail, Result};
use colored::Colorize as _;
use indicatif::ProgressBar;
use libium::{
    config::structs::{
        Filters, ModLoader, Profile, ProfileItem, Source, SourceId, SourceKind, Version,
    },
    upgrade::{mod_downloadable, DownloadData},
};
use parking_lot::Mutex;
use std::{
    fs::read_dir,
    mem::take,
    sync::{mpsc, Arc},
    time::Duration,
};
use tokio::task::JoinSet;

/// Get the latest compatible downloadable for the sources in `profile`
///
/// If an error occurs with a resolving task, instead of failing immediately,
/// resolution will continue and the error return flag is set to true.
async fn get_platform_downloadables(
    kind: SourceKind,
    profile: &Profile,
    filters: &Filters,
) -> Result<(Vec<DownloadData>, bool)> {
    let progress_bar = Arc::new(Mutex::new(ProgressBar::new(0).with_style(STYLE_NO.clone())));
    let mut tasks = JoinSet::new();
    let mut done_sources = Vec::new();
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

            tasks.spawn(async move {
                let permit = SEMAPHORE.get_or_init(default_semaphore).acquire().await?;

                let result = source.fetch_download_file(vec![&filters]).await;

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
                        Ok(Some(download_file))
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
    let to_download = tasks.into_iter().flatten().collect();

    Ok((to_download, error))
}

pub async fn upgrade(
    profile_item: &ProfileItem,
    profile: &Profile,
    filters: Filters,
) -> Result<()> {
    let filters = filters.concat(profile.filters.clone());

    check_unstrict_filter(&filters);

    for kind in SourceKind::ARRAY {
        if !profile.map(*kind).is_empty() {
            println!(
                "{} {}",
                "Upgrading".bold(),
                match kind {
                    SourceKind::Mods => "Mods",
                    SourceKind::Resourcepacks => "Resourcepacks",
                    SourceKind::Shaders => "Shaders",
                    SourceKind::Modpacks => "Modpacks",
                }
                .bold()
            );

            upgrade_inner(*kind, profile_item, profile, &filters).await?;
        }
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
            "Warning: specified multiple incompatible mod loaders"
                .yellow()
                .bold()
        );
    }
}

async fn upgrade_inner(
    kind: SourceKind,
    profile_item: &ProfileItem,
    profile: &Profile,
    filters: &Filters,
) -> Result<()> {
    let dir = match kind {
        SourceKind::Mods => &profile_item.mods_dir,
        SourceKind::Resourcepacks => &profile_item.resourcepacks_dir,
        SourceKind::Shaders => &profile_item.shaderpacks_dir,
        SourceKind::Modpacks => todo!("modpacks are not yet supported"),
    };

    let (mut to_download, error) = get_platform_downloadables(kind, profile, filters).await?;
    let mut to_install = Vec::new();
    if dir.join("user").exists()
        && filters.mod_loaders.as_ref().and_then(|x| x.first()) != Some(&ModLoader::Quilt)
    {
        for file in read_dir(dir.join("user"))? {
            let file = file?;
            let path = file.path();
            if path.is_file()
                && path
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("jar"))
            {
                to_install.push((file.file_name(), path));
            }
        }
    }

    clean(dir, &mut to_download, &mut to_install).await?;
    to_download
        .iter_mut()
        // Download directly to the output directory
        .map(|thing| thing.output = thing.filename().into())
        .for_each(drop); // Doesn't drop any data, just runs the iterator
    if to_download.is_empty() && to_install.is_empty() {
        println!("\n{}", "All up to date!".bold());
    } else {
        println!(
            "\n{}{}{}\n",
            "Downloading ".bold(),
            match kind {
                SourceKind::Mods => "Mod",
                SourceKind::Resourcepacks => "Resourcepack",
                SourceKind::Shaders => "Shader",
                SourceKind::Modpacks => "Modpack",
            },
            " Files".bold()
        );
        download(dir.clone(), Some(profile_item), to_download, to_install).await?;
    }

    if error {
        Err(anyhow!(
            "\nCould not get the latest compatible version of some {}",
            match kind {
                SourceKind::Mods => "mods",
                SourceKind::Resourcepacks => "resourcepacks",
                SourceKind::Shaders => "shaderpacks",
                SourceKind::Modpacks => "modpacks",
            }
        ))
    } else {
        Ok(())
    }
}
