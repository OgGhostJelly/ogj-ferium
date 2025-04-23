use std::io::{Read, Seek};

use zip::{result::ZipResult, ZipArchive};

pub use zip_extensions::zip_extract;

use crate::read_wrapper;

pub mod curseforge;
pub mod modrinth;

/// Returns the contents of the `file_name` from the provided `input` zip file if it exists
pub fn read_file_from_zip(input: impl Read + Seek, file_name: &str) -> ZipResult<Option<String>> {
    let mut zip_file = ZipArchive::new(input)?;

    let ret = if let Ok(entry) = zip_file.by_name(file_name) {
        Ok(Some(read_wrapper(entry)?))
    } else {
        Ok(None)
    };
    ret
}
