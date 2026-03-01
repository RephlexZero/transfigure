#[derive(Clone, PartialEq)]
pub enum FileStatus {
    Pending,
    Converting,
    Done(Vec<u8>),
    Error(String),
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
