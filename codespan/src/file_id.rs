use std::fmt::Display;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId(u32);

impl FileId {
    const OFFSET: u32 = 1;
    pub const NONE: Self = FileId(0);

    pub fn new(index: usize) -> FileId {
        FileId(index as u32 + Self::OFFSET)
    }

    pub fn get(self) -> usize {
        (self.0 - Self::OFFSET) as usize
    }
}

impl Display for FileId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
