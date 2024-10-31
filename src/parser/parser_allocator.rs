use core::{mem::MaybeUninit, str::from_utf8};

use bump_into::BumpInto;

#[derive(Debug)]
pub enum AllocError {
    OutOfMemory,
    Utf8Error,
}

pub struct ParserAllocator<'b>(BumpInto<'b>);

impl<'b> ParserAllocator<'b> {
    pub fn new(slice: &'b mut [MaybeUninit<u8>]) -> Self {
        Self(BumpInto::from_slice(slice))
    }

    pub fn alloc<T>(&self, t: T) -> Result<&'b mut T, AllocError> {
        self.0.alloc(t).map_err(|_| AllocError::OutOfMemory)
    }

    pub fn alloc_str(&self, s: &str) -> Result<&'b str, AllocError> {
        self.0
            .alloc_copy_concat_strs(&[s])
            .ok_or(AllocError::OutOfMemory)
            .map(|s| &*s)
    }
    pub fn alloc_str_from_bytes<'a>(&self, bytes: &'a [u8]) -> Result<&'b str, AllocError> {
        let as_str = from_utf8(bytes).map_err(|_| AllocError::Utf8Error)?;
        self.alloc_str(as_str)
    }
}
