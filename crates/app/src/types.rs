use std::sync::Arc;

// Byte buffers are reference-counted so cloning the file list (which the
// reactive system does on every update) never copies file contents.

#[derive(Clone, PartialEq)]
pub enum FileStatus {
    Pending,
    Converting,
    Done { data: Arc<[u8]>, elapsed_ms: u32 },
    Error(String),
}

impl FileStatus {
    pub fn is_finished(&self) -> bool {
        matches!(self, FileStatus::Done { .. } | FileStatus::Error(_))
    }
}

#[derive(Clone, PartialEq)]
pub struct BatchFile {
    pub id: usize,
    pub name: String,
    pub bytes: Arc<[u8]>,
    pub extension: String,
    /// Output formats this file can become, most useful first.
    pub formats: Vec<&'static str>,
    pub size: usize,
    pub target: Option<String>,
    pub status: FileStatus,
}

/// Where a file is in its conversion, without the output bytes, so views
/// can be compared and cloned cheaply.
#[derive(Clone, PartialEq, Debug)]
pub enum Phase {
    Pending,
    Converting,
    Done { out_size: usize, elapsed_ms: u32 },
    Failed(String),
}

impl From<&FileStatus> for Phase {
    fn from(status: &FileStatus) -> Self {
        match status {
            FileStatus::Pending => Phase::Pending,
            FileStatus::Converting => Phase::Converting,
            FileStatus::Done { data, elapsed_ms } => Phase::Done {
                out_size: data.len(),
                elapsed_ms: *elapsed_ms,
            },
            FileStatus::Error(e) => Phase::Failed(e.clone()),
        }
    }
}

/// What the interface shows for one file.
#[derive(Clone, PartialEq, Debug)]
pub struct FileView {
    pub id: usize,
    /// 1-based position in the queue.
    pub number: usize,
    pub name: String,
    pub ext: String,
    pub size: usize,
    pub formats: Vec<&'static str>,
    pub target: Option<String>,
    pub phase: Phase,
}

impl FileView {
    pub fn from_file(number: usize, f: &BatchFile) -> Self {
        // Formats the converter doesn't know still show their own extension.
        let ext = if f.extension.is_empty() {
            f.name
                .rsplit_once('.')
                .filter(|(stem, _)| !stem.is_empty())
                .map(|(_, ext)| ext.to_lowercase())
                .unwrap_or_default()
        } else {
            f.extension.clone()
        };
        Self {
            id: f.id,
            number,
            name: f.name.clone(),
            ext,
            size: f.size,
            formats: f.formats.clone(),
            target: f.target.clone(),
            phase: Phase::from(&f.status),
        }
    }

    pub fn is_busy(&self) -> bool {
        self.phase == Phase::Converting
    }
}
