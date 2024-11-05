// use alloc::{borrow::ToOwned, boxed::Box};
// use core::str::from_utf8;

// #[derive(Debug)]
// pub enum AllocError {
//     OutOfMemory,
//     Utf8Error,
// }

// pub struct ParserAllocator {
//     strings: alloc::vec::Vec<alloc::string::String>,
// }

// impl ParserAllocator {
//     pub fn new() -> Self {
//         Self {
//             strings: Default::default(),
//         }
//     }

//     pub fn alloc<T>(&self, t: T) -> Result<Box<T>, AllocError> {
//         Ok(Box::new(t))
//     }

//     // pub fn alloc_str_space(&self, len: usize) -> Result<&'b mut [u8], AllocError> {
//     //     bump_into::space_uninit!(len)
//     //         .alloc_n_with(len, core::iter::repeat(0u8))
//     //         .map_err(|_| AllocError::OutOfMemory)
//     // }

//     // pub fn alloc_str(&mut self, s: &str) -> Result<String, AllocError> {
//     //     if let Some(s) = self.strings.iter().find(|owned| &owned[..] == s) {
//     //         return Ok(s);
//     //     }

//     //     let owned = alloc::string::String::from(s);
//     //     self.strings.push(owned);
//     //     let sr = self.strings.last().unwrap();
//     //     Ok(sr)
//     // }

//     pub fn alloc_str_from_bytes(
//         &mut self,
//         bytes: &[u8],
//     ) -> Result<alloc::string::String, AllocError> {
//         let as_str = from_utf8(bytes).map_err(|_| AllocError::Utf8Error)?;
//         Ok(as_str.to_owned())
//     }
// }
