use super::*;
use failure::Fail;
use ioutil::*;

pub trait Error: Fail {
    fn errno(&self) -> i32;
}

pub trait Node<'a>: Sized {
    type Error: Error;

    fn file_attributes(&self, ino: u64) -> fuse::FileAttr;

    fn open_ro(&self) -> Result<Box<ReadSeek + 'a>, Self::Error>;

    fn children(&self) -> Result<Vec<(String, Self)>, Self::Error>;
}
