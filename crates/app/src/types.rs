#[derive(Clone, PartialEq)]
pub enum FileStatus {
    Pending,
    Converting,
    Done { data: Vec<u8>, elapsed_ms: u32 },
    Error(String),
}

impl FileStatus {
    pub fn is_finished(&self) -> bool {
        matches!(self, FileStatus::Done { .. } | FileStatus::Error(_))
    }
}

#[derive(Clone)]
pub struct BatchFile {
    pub id: usize,
    pub name: String,
    pub bytes: Vec<u8>,
    pub extension: String,
    pub size: usize,
    pub target: Option<String>,
    pub status: FileStatus,
}
