use std::io::{Cursor, Write};

/// A single file entry for archive creation.
pub struct ArchiveEntry {
    pub name: String,
    pub data: Vec<u8>,
}

/// Create a ZIP archive from the given entries.
pub fn create_zip(entries: &[ArchiveEntry]) -> Result<Vec<u8>, String> {
    let buf = Cursor::new(Vec::new());
    let mut zip = zip::ZipWriter::new(buf);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for entry in entries {
        zip.start_file(&entry.name, options)
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

    for entry in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(entry.data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, &entry.name, &*entry.data)
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

    for entry in entries {
        let mut header = tar::Header::new_gnu();
        header.set_size(entry.data.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        tar.append_data(&mut header, &entry.name, &*entry.data)
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
