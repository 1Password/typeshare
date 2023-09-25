use crate::ffi_interop::ffi_v1::FFIArray;

use typeshare_core::parsed_types::ParsedData;

#[repr(transparent)]
pub struct RawParsedData(FFIArray<u8>);

impl RawParsedData {
    pub fn new(parsed_data: ParsedData) -> Self {
        let parsed_data =
            bincode::serde::encode_to_vec(parsed_data, bincode::config::standard()).unwrap();
        Self(parsed_data.try_into().unwrap())
    }

    pub fn into_parsed_data(self) -> ParsedData {
        let reader = raw_parsed_data_reader::FFIArrayReader::new(self.0);
        bincode::serde::decode_from_reader::<ParsedData, _, _>(reader, bincode::config::standard())
            .unwrap()
    }
}
mod raw_parsed_data_reader {
    use crate::ffi_interop::ffi_v1::FFIArray;
    use bincode::de::read::Reader;
    use bincode::error::DecodeError;
    use std::slice;

    pub struct FFIArrayReader {
        ffi_array: Box<[u8]>,
    }
    impl Reader for FFIArrayReader {
        #[inline]
        fn read(&mut self, bytes: &mut [u8]) -> Result<(), DecodeError> {
            if bytes.len() > self.ffi_array.len() {
                return Err(DecodeError::UnexpectedEnd {
                    additional: bytes.len() - self.ffi_array.len(),
                });
            }
            let (read_slice, remaining) = self.ffi_array.split_at_mut(bytes.len());
            bytes.copy_from_slice(read_slice);
            unsafe {
                self.ffi_array = Box::from_raw(remaining);
            }

            Ok(())
        }
        #[inline]
        fn peek_read(&mut self, n: usize) -> Option<&[u8]> {
            self.ffi_array.get(..n)
        }

        #[inline]
        fn consume(&mut self, n: usize) {
            // FFI Array = [1, 2, 3, 4, 5] (len 4 )
            // Consume 5
            if n >= (self.ffi_array.len() + 1) {
                self.ffi_array = Box::new([]);
            }
            unsafe {
                let remaining = self.ffi_array.get_unchecked_mut(n..);
                self.ffi_array = Box::from_raw(remaining);
            }
        }
    }
    impl FFIArrayReader {
        /// Creates a Reader from an FFIArray
        pub fn new(ffi_array: FFIArray<u8>) -> Self {
            unsafe {
                let (ptr, size) = ffi_array.into_inner();
                let boxed_slice = slice::from_raw_parts_mut(ptr, size);
                let boxed_slice = Box::from_raw(boxed_slice);
                Self {
                    ffi_array: boxed_slice,
                }
            }
        }
    }
}
