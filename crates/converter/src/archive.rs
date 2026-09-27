use std::io::{Cursor, Write};

/// A single file entry for archive creation.
pub struct ArchiveEntry {
    pub name: String,
    pub data: Vec<u8>,
}

/// Entry names made unique (case-insensitively, as on Windows and macOS):
/// `photo.webp`, `photo (2).webp`, … Two inputs can convert to the same
/// output name, and archive writers reject or silently overwrite duplicates.
fn unique_names(entries: &[ArchiveEntry]) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    entries
        .iter()
        .map(|e| {
            let (stem, ext) = match e.name.rsplit_once('.') {
                Some((s, x)) if !s.is_empty() => (s.to_string(), format!(".{x}")),
                _ => (e.name.clone(), String::new()),
            };
            let mut name = e.name.clone();
            let mut n = 2;
            while !seen.insert(name.to_lowercase()) {
                name = format!("{stem} ({n}){ext}");
                n += 1;
            }
            name
        })
        .collect()
}

/// Create a ZIP archive from the given entries.
pub fn create_zip(entries: &[ArchiveEntry]) -> Result<Vec<u8>, String> {
    let buf = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for (entry, name) in entries.iter().zip(unique_names(entries)) {
        zip.start_file(name, options)
            .map_err(|e| format!("zip error: {e}"))?;
        zip.write_all(&entry.data)
            .map_err(|e| format!("zip write error: {e}"))?;
    }

    let cursor = zip.finish().map_err(|e| format!("zip finish error: {e}"))?;
    Ok(cursor.into_inner())
}

/// Create a tar.gz archive from the given entries.
pub fn create_tar_gz(entries: &[ArchiveEntry]) -> Result<Vec<u8>, String> {
    let buf = Vec::new();
    let encoder = flate2::write::GzEncoder::new(buf, flate2::Compression::default());
    let mut tar = tar::Builder::new(encoder);

    for (entry, name) in entries.iter().zip(unique_names(entries)) {
        let mut header = tar::Header::new_gnu();
        header.set_size(entry.data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, &name, &*entry.data)
            .map_err(|e| format!("tar error: {e}"))?;
    }

    let encoder = tar
        .into_inner()
        .map_err(|e| format!("tar finish error: {e}"))?;
    let compressed = encoder
        .finish()
        .map_err(|e| format!("gzip finish error: {e}"))?;
    Ok(compressed)
}

/// Create a tar.xz archive from the given entries.
pub fn create_tar_xz(entries: &[ArchiveEntry]) -> Result<Vec<u8>, String> {
    // First create an uncompressed tar
    let buf = Vec::new();
    let mut tar = tar::Builder::new(buf);

    for (entry, name) in entries.iter().zip(unique_names(entries)) {
        let mut header = tar::Header::new_gnu();
        header.set_size(entry.data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, &name, &*entry.data)
            .map_err(|e| format!("tar error: {e}"))?;
    }

    let tar_bytes = tar
        .into_inner()
        .map_err(|e| format!("tar finish error: {e}"))?;

    // Then compress with LZMA/XZ
    let mut compressed = Vec::new();
    lzma_rs::xz_compress(&mut Cursor::new(&tar_bytes), &mut compressed)
        .map_err(|e| format!("xz compress error: {e}"))?;
    Ok(compressed)
}

/// Create a 7z archive from the given entries.
pub fn create_7z(entries: &[ArchiveEntry]) -> Result<Vec<u8>, String> {
    let buf = Cursor::new(Vec::new());
    let mut writer =
        sevenz_rust2::ArchiveWriter::new(buf).map_err(|e| format!("7z create error: {e}"))?;

    for (entry, name) in entries.iter().zip(unique_names(entries)) {
        let archive_entry = sevenz_rust2::ArchiveEntry::new_file(&name);
        let reader = Cursor::new(&entry.data);
        writer
            .push_archive_entry(archive_entry, Some(reader))
            .map_err(|e| format!("7z entry error: {e}"))?;
    }

    let cursor = writer
        .finish()
        .map_err(|e| format!("7z finish error: {e}"))?;
    Ok(cursor.into_inner())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duplicate_names_get_suffixes() {
        let e = |n: &str| ArchiveEntry {
            name: n.into(),
            data: vec![],
        };
        let names = unique_names(&[e("a.webp"), e("A.webp"), e("a.webp"), e("README")]);
        assert_eq!(names, ["a.webp", "A (2).webp", "a (3).webp", "README"]);
    }

    #[test]
    fn zip_with_duplicate_names_succeeds() {
        let e = |n: &str| ArchiveEntry {
            name: n.into(),
            data: b"x".to_vec(),
        };
        assert!(create_zip(&[e("a.webp"), e("a.webp")]).is_ok());
    }
}
