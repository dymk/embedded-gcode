use core::str::from_utf8;

use bump_into::BumpInto;

pub enum AllocError {
    OutOfMemory,
    Utf8Error,
}

#[derive(Clone, Copy)]
pub struct NomAlloc<'b>(&'b BumpInto<'b>);

impl<'b> NomAlloc<'b> {
    pub fn new(bump: &'b BumpInto<'b>) -> Self {
        Self(bump)
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
