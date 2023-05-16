use std::str::Utf8Error;
use crate::blk::error::ParseError;

#[derive(Debug, thiserror::Error)]
pub enum VromfError {
	#[error("Expected buffer of length {expected_size}, found {found_buff}")]
	InvalidIntegerBuffer {
		expected_size: usize,
		found_buff:    usize,
	},

	#[error("{found} is not a valid header")]
	InvalidHeaderType { found: u32 },

	#[error("{found:X} is not a valid digest-header")]
	DigestHeader { found: u8 },

	#[error("{found} is not a valid platform-type")]
	InvalidPlatformType { found: u32 },

	#[error("{found:X} is not a valid vromf-packing-configuration")]
	InvalidPackingConfiguration { found: u8 },

	#[error("current ptr {current_ptr} + {requested_len} bytes are out of bounds for file of size: {file_size}")]
	IndexingFileOutOfBounds {
		current_ptr:   usize,
		file_size:     usize,
		requested_len: usize,
	},

	#[error("Could not parse usize from u64: {from}, because usize may exactly hold {} bytes", std::mem::size_of::< usize > ())]
	UsizeFromU64 { from: u64 },

	#[error("Unaligned chunks: the data-set of size {len} was supposed to align/chunk into {align}, but {rem} remained")]
	UnalignedChunks {
		len:   usize,
		align: usize,
		rem:   usize,
	},

	#[error("{}", fmt_utf8_error(buff, utf8e))]
	Utf8 { buff: Vec<u8>, utf8e: Utf8Error },

	#[error("Name map parsing. IF THIS OCCURS: OPEN AN ISSUE OR MESSAGE \"FlareFlo🦆#2800\" on discord")] // If this gets thrown, i should consider properly catching errors in the NM functions
	InvalidNm,

	#[error("Failed because {file_name} requires a name-map, but the Vromf did not contain one")]
	MissingNm {
		file_name: String,
	},

	#[error("Failed because {file_name} requires a dictionary, but the Vromf did not contain one")]
	MissingDict {
		file_name: String,
	},

	#[error(transparent)]
	BlkParseError(#[from] ParseError),

	#[error("Failed to cherry pick file {path_name}, it is probably missing?")]
	FileNotInVromf {
		path_name: String,
	},

	#[error(transparent)]
	Fmt(#[from] std::fmt::Error),
}

fn fmt_utf8_error(buff: &Vec<u8>, e: &Utf8Error) -> String {
	if let Some(len) = e.error_len() {
		let invalid = &buff[e.valid_up_to()..len];
		format!("The sequence {invalid:?} is not a valid UTF-8 sequence, byte offset into buffer: {}..{len}", e.valid_up_to())
	} else {
		let invalid = &buff[e.valid_up_to()..];
		format!("Buffer terminated within UTF-8 sequence {invalid:?} was ended early")
	}
}
