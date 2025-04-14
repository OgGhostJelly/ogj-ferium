use crate::{default_semaphore, warn, SEMAPHORE, STYLE_BYTE, TICK};
use anyhow::{anyhow, bail, Error, Result};
use colored::Colorize as _;
use fs_extra::{
    dir::{copy as copy_dir, CopyOptions as DirCopyOptions},
    file::{move_file, CopyOptions as FileCopyOptions},
};
use indicatif::ProgressBar;
use libium::{iter_ext::IterExt as _, upgrade::DownloadData};
use parking_lot::Mutex;
use std::{
    ffi::OsString,
    fs::{copy, create_dir_all, read_dir, remove_file},
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};
use tokio::task::JoinSet;

/// Check the given `directory`
///
/// - If there are files there that are not in `to_download` or `to_install`, they will be moved to `directory`/.old
/// - If a file in `to_download` or `to_install` is already there, it will be removed from the respective vector
/// - If the file is a `.part` file or if the move failed, the file will be deleted
pub async fn clean(
    directory: &Path,
    to_download: &mut Vec<DownloadData>,
    to_install: &mut Vec<InstallData>,
    move_old: bool,
) -> Result<()> {
    let dupes = find_dupes_by_key(to_download, DownloadData::filename);
    if !dupes.is_empty() {
        warn!(
            "{} duplicate files were found {}. Remove the mod it belongs to",
            dupes.len(),
            dupes
                .into_iter()
                .map(|i| to_download.swap_remove(i).filename())
                .display(", ")
        );
    }
    if move_old {
        create_dir_all(directory.join(".old"))?;
    }
    for file in read_dir(directory)? {
        let file = file?;
        // If it's a file
        if file.file_type()?.is_file() {
            let filename = file.file_name();
            let filename = filename.to_string_lossy();
            let filename = filename.as_ref();
            // If it is already downloaded
            if let Some(index) = to_download
                .iter()
                .position(|thing| filename == thing.filename())
            {
                // Don't download it
                to_download.swap_remove(index);
            // Likewise, if it is already installed
            } else if let Some(index) = to_install.iter().position(|target| filename == target.to.1)
            {
                // Don't install it
                to_install.swap_remove(index);
            // Or else, move the file to `directory`/.old
            // If the file is a `.part` file or if the move failed, delete the file
            } else if filename.ends_with("part")
                || (move_old
                    && move_file(
                        file.path(),
                        directory.join(".old").join(filename),
                        &FileCopyOptions::new(),
                    )
                    .is_err())
            {
                remove_file(file.path())?;
            }
        }
    }
    Ok(())
}

/// Download and install the files in `to_download` and `to_install` to the paths set in `profile`
pub async fn download(
    minecraft_dir: PathBuf,
    to_download: Vec<DownloadData>,
    to_install: Vec<InstallData>,
) -> Result<()> {
    let progress_bar = Arc::new(Mutex::new(
        ProgressBar::new(
            to_download
                .iter()
                .map(|downloadable| downloadable.length as u64)
                .sum(),
        )
        .with_style(STYLE_BYTE.clone()),
    ));
    progress_bar
        .lock()
        .enable_steady_tick(Duration::from_millis(100));
    let mut tasks = JoinSet::new();
    let client = reqwest::Client::new();

    for downloadable in to_download {
        let progress_bar = Arc::clone(&progress_bar);
        let client = client.clone();
        let minecraft_dir = minecraft_dir.clone();

        tasks.spawn(async move {
            let _permit = SEMAPHORE.get_or_init(default_semaphore).acquire().await?;

            let (length, filename) = downloadable
                .download(client, minecraft_dir, |additional| {
                    progress_bar.lock().inc(additional as u64);
                })
                .await?;
            progress_bar.lock().println(format!(
                "{} Downloaded  {:>7}  {}",
                &*TICK,
                size::Size::from_bytes(length)
                    .format()
                    .with_base(size::Base::Base10)
                    .to_string(),
                filename.dimmed(),
            ));
            Ok::<(), Error>(())
        });
    }
    for res in tasks.join_all().await {
        res?;
    }
    Arc::try_unwrap(progress_bar)
        .map_err(|_| anyhow!("Failed to run threads to completion"))?
        .into_inner()
        .finish_and_clear();
    for target in to_install {
        let InstallData {
            from: path,
            to: (output_dir, name),
        } = target;

        let output_dir = minecraft_dir.join(output_dir);
        if path.is_file() {
            copy(path, output_dir.join(&name))?;
        } else if path.is_dir() {
            let mut copy_options = DirCopyOptions::new();
            copy_options.overwrite = true;
            copy_dir(path, &output_dir, &copy_options)?;
        } else {
            bail!("Could not determine whether installable is a file or folder")
        }
        println!(
            "{} Installed          {}",
            &*TICK,
            name.to_string_lossy().dimmed()
        );
    }

    Ok(())
}

pub struct InstallData {
    from: PathBuf,
    to: (PathBuf, OsString),
}

/// Construct a `to_install` vector from the `directory`
pub fn read_overrides(to_install: &mut Vec<InstallData>, directory: &Path) -> Result<()> {
    if directory.exists() {
        for file in read_dir(directory)? {
            let file = file?;
            let from = file.path();

            let Some(to) = from.parent() else {
                bail!("override path should have a parent")
            };
            let Ok(to) = to.strip_prefix(directory) else {
                bail!("override path should have the overrides directory as a prefix")
            };

            to_install.push(InstallData {
                from: file.path(),
                to: (to.to_path_buf(), file.file_name()),
            });
        }
    }
    Ok(())
}

/// Find duplicates of the items in `slice` using a value obtained by the `key` closure
///
/// Returns the indices of duplicate items in reverse order for easy removal
fn find_dupes_by_key<T, V, F>(slice: &mut [T], key: F) -> Vec<usize>
where
    V: Eq + Ord,
    F: Fn(&T) -> V,
{
    let mut indices = Vec::new();
    if slice.len() < 2 {
        return indices;
    }
    slice.sort_unstable_by_key(&key);
    for i in 0..(slice.len() - 1) {
        if key(&slice[i]) == key(&slice[i + 1]) {
            indices.push(i);
        }
    }
    indices.reverse();
    indices
}
