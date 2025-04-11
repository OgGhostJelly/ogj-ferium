use crate::{default_semaphore, SEMAPHORE, STYLE_BYTE, TICK};
use anyhow::{anyhow, Error, Result};
use colored::Colorize as _;
use fs_extra::file::{move_file, CopyOptions as FileCopyOptions};
use indicatif::ProgressBar;
use libium::{config::structs::SourceKind, iter_ext::IterExt as _, upgrade::DownloadData};
use parking_lot::Mutex;
use std::{
    fs::{create_dir_all, read_dir, remove_file},
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
    to_download: &mut Vec<(SourceKind, DownloadData)>,
) -> Result<()> {
    let dupes = find_dupes_by_key(to_download, |(_, d)| d.filename());
    if !dupes.is_empty() {
        println!(
            "{}",
            format!(
                "Warning: {} duplicate files were found {}. Remove the mod it belongs to",
                dupes.len(),
                dupes
                    .into_iter()
                    .map(|i| to_download.swap_remove(i).1.filename())
                    .display(", ")
            )
            .yellow()
            .bold()
        );
    }
    create_dir_all(directory.join(".old"))?;
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
                .position(|(_, thing)| filename == thing.filename())
            {
                // Don't download it
                to_download.swap_remove(index);
            // Or else, move the file to `directory`/.old
            // If the file is a `.part` file or if the move failed, delete the file
            } else if filename.ends_with("part")
                || move_file(
                    file.path(),
                    directory.join(".old").join(filename),
                    &FileCopyOptions::new(),
                )
                .is_err()
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
    to_download: Vec<(SourceKind, DownloadData)>,
) -> Result<()> {
    let progress_bar = Arc::new(Mutex::new(
        ProgressBar::new(
            to_download
                .iter()
                .map(|(_, downloadable)| downloadable.length as u64)
                .sum(),
        )
        .with_style(STYLE_BYTE.clone()),
    ));
    progress_bar
        .lock()
        .enable_steady_tick(Duration::from_millis(100));
    let mut tasks = JoinSet::new();
    let client = reqwest::Client::new();

    for (kind, downloadable) in to_download {
        let progress_bar = Arc::clone(&progress_bar);
        let client = client.clone();

        let Some(output_dir) = kind.directory(&minecraft_dir) else {
            todo!("modpacks are not supported yet!");
        };

        tasks.spawn(async move {
            let _permit = SEMAPHORE.get_or_init(default_semaphore).acquire().await?;

            let (length, filename) = downloadable
                .download(client, &output_dir, |additional| {
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
