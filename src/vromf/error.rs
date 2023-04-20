use std::string::FromUtf8Error;

#[derive(Debug, thiserror::Error)]
pub enum VromfError {
    #[error("Expected buffer of length {expected_size}, found {found_buff}")]
    InvalidIntegerBuffer {
        expected_size: usize,
        found_buff: usize,
    },

    #[error("{found} is not a valid header")]
    InvalidHeaderType {
        found: u32,
    },

    #[error("{found:X} is not a valid digest-heaader")]
    DigestHeader {
        found: u8
    },

    #[error("{found} is not a valid platform-type")]
    InvalidPlatformType {
        found: u32
    },

    #[error("{found:X} is not a valid vromf-packing-configuration")]
    InvalidPackingConfiguration {
        found: u8,
    },

    #[error("current ptr {current_ptr} + {requested_len} bytes are out of bounds for file of size: {file_size}")]
    IndexingFileOutOfBounds {
        current_ptr: usize,
        file_size: usize,
        requested_len: usize,
    },

    #[error("Could not parse usize from u64: {from}, because usize may exactly hold {} bytes", std::mem::size_of::<usize>())]
    UsizeFromU64 {
        from: u64,
    },

    #[error("Unaligned chunks: the data-set of size {len} was supposed to align/chunk into {align}, but {rem} remained")]
    UnalignedChunks {
        len: usize,
        align: usize,
        rem: usize,
    },

    #[error("Invalid UTF-8 string: {invalid}")]
    Utf8{
        invalid: String,
    },
}