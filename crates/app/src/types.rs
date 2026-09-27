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
    pub size: usize,
    pub target: Option<String>,
    pub status: FileStatus,
}
