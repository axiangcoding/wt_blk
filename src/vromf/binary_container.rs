use std::mem::size_of;
use color_eyre::eyre::{Context, eyre};
use color_eyre::{Report, Section};
use crate::util::{debug_hex, format_hex};

use crate::vromf::{
	de_obfuscation::deobfuscate,
	enums::{HeaderType, PlatformType},
	util::{bytes_to_int, pack_type_from_aligned},
};

pub(crate) fn decode_bin_vromf(file: &[u8]) -> Result<Vec<u8>, Report> {
	let mut ptr = 0_usize;

	// Returns slice offset from file, incrementing the ptr by offset
	let idx_file_offset = |ptr: &mut usize, offset: usize| {
		if let Some(buff) = file.get(*ptr..(*ptr + offset)) {
			*ptr += offset;
			Ok(buff)
		} else {
			Err(eyre!("Indexing buffer of size {} with index {} and length {}", file.len(), *ptr, offset))
		}
	};

	let header_type = bytes_to_int(idx_file_offset(&mut ptr, 4)?)?;
	let header_type = HeaderType::try_from(header_type)?;

	let platform_raw = bytes_to_int(idx_file_offset(&mut ptr, 4)?)?;
	let _platform = PlatformType::try_from(platform_raw)?;

	// Size of the file before compression
	let size = bytes_to_int(idx_file_offset(&mut ptr, 4)?)?;

	let header_packed: u32 = bytes_to_int(idx_file_offset(&mut ptr, 4)?)?;

	// Type of compression/packing, and size before compression
	let (pack_type, extended_header_size) = pack_type_from_aligned(header_packed)?;

	let inner_data = if header_type.is_extended() {
		let extended_header = idx_file_offset(
			&mut ptr,
			size_of::<u16>() + size_of::<u16>() + size_of::<u32>(),
		)?;
		let s = extended_header; // Copying ptr such that indexing below is less verbose

		// Unused header elements, for now
		let _header_size = u16::from_le_bytes([s[0], s[1]]);
		let _flags = u16::from_le_bytes([s[2], s[3]]);
		// The version is always reversed in order. It may never exceed 255
		let _version = (s[7], s[6], s[5], s[4]);

		idx_file_offset(&mut ptr, extended_header_size as usize)?
	} else {
		if pack_type.is_compressed() {
			idx_file_offset(&mut ptr, extended_header_size as usize)?
		} else {
			idx_file_offset(&mut ptr, size as usize)?
		}
	};

	// Directly return when data is not obfuscated
	if !pack_type.is_obfuscated() {
		return Ok(inner_data.to_vec());
	}

	let mut output = inner_data.to_vec();
	deobfuscate(&mut output);

	if pack_type.is_compressed() {
		output = zstd::decode_all(output.as_slice()).note("This most likely occurred because of improper computation of the frame-size")?;
	}

	Ok(output)
}

#[cfg(test)]
mod test {
	use std::fs;
	use std::path::PathBuf;
	use std::str::FromStr;

	use crate::vromf::binary_container::decode_bin_vromf;
	use crate::vromf::unpacker::VromfUnpacker;

	#[test]
	fn decode_simple() {
		let f = fs::read("./samples/checked_simple_uncompressed_checked.vromfs.bin").unwrap();
		decode_bin_vromf(&f).unwrap();
	}

	#[test]
	fn decode_compressed() {
		let f = fs::read("./samples/unchecked_extended_compressed_checked.vromfs.bin").unwrap();
		decode_bin_vromf(&f).unwrap();
	}

	// #[test]
	// fn test_regional() {
	// 	let f = fs::read("./samples/regional.vromfs.bin").unwrap();
	// 	// decode_bin_vromf(&f).unwrap();
	// 	let unpacker = VromfUnpacker::from_file((PathBuf::from_str("asas").unwrap(), f)).unwrap();
	// 	unpacker.unpack_all(None).unwrap();
	// }
}
