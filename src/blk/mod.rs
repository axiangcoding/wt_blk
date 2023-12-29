use std::{
	fs,
	fs::ReadDir,
	sync::{
		Arc,
		atomic::{AtomicUsize, Ordering},
	},
};

pub use ::zstd::dict::DecoderDictionary;
use cfg_if::cfg_if;
use color_eyre::eyre::{Context, ContextCompat};
use color_eyre::Report;

use crate::blk::{
	file::FileType,
	nm_file::NameMap,
	parser::parse_blk,
	zstd::{BlkDecoder, decode_zstd},
};
use crate::blk::blk_structure::BlkField;
use crate::blk::blk_type::BlkType;
use crate::blk::util::blk_str;
use crate::vromf::BlkOutputFormat;

/// Decodes flat map of fields into the corresponding nested datastructure
mod blk_block_hierarchy;

/// Defines the recursive/nested structure that BLK files are represented with internally
pub mod blk_structure;

/// Defines the primitive types that BLK stores
pub mod blk_type;

/// Shared error that is returned from hot functions,
/// otherwise, [`color_eyre::Report`] is used
pub mod error;

/// One-byte file header that each blk file begins with
pub mod file;

/// Utility function to decode ULEB128 encoded files
/// <https://en.wikipedia.org/wiki/LEB128>
pub mod leb128;

/// Struct storing a shared map of strings that multiple BLK files reference
pub mod nm_file;

/// Exports core function for unpacking BLK file
pub mod parser;

cfg_if! {
	if #[cfg(test)] {
		/// Unit tests
		pub mod test;
	}
}

/// Collection of macros and functions used in all BLK modules
pub mod util;

/// Zstandard unpacking functionality
pub mod zstd;
mod repack;

/// Implementations for serializing into human readable text formats
pub mod plaintext_serialize;

/// Implementations for deserializing into BLk binary format from BlkText
mod plaintext_deserialize;

/// Utility struct which wraps
mod blk_decoder;

#[allow(dead_code)]
fn test_parse_dir(
	pile: &mut Vec<(String, Vec<u8>)>,
	dir: ReadDir,
	total_files_processed: &AtomicUsize,
) {
	for file in dir {
		let file = file.as_ref().unwrap();
		if file.metadata().unwrap().is_dir() {
			test_parse_dir(pile, file.path().read_dir().unwrap(), total_files_processed);
		} else {
			let fname = file.file_name().to_str().unwrap().to_owned();
			if fname.ends_with(".blk") {
				let read = fs::read(file.path()).unwrap();
				pile.push((fname, read));
				total_files_processed.fetch_add(1, Ordering::Relaxed);
			}
		}
	}
}

/// Highest-level function for unpacking one BLK explicitly, for direct low level control call [`parser::parse_blk`]
pub fn unpack_blk(mut file: Vec<u8>, dictionary: Option<&BlkDecoder>, nm: Option<Arc<NameMap>>) -> Result<BlkField, Report> {
	let mut offset = 0;
	let file_type = FileType::from_byte(file[0])?;
	if file_type.is_zstd() {
		if file_type == FileType::FAT_ZSTD { offset += 1 }; // FAT_ZSTD has a leading byte indicating that its unpacked form is of the FAT format
		file = decode_zstd(file_type, &file, dictionary)?;
	} else {
		// uncompressed Slim and Fat files retain their initial magic bytes
		offset = 1;
	};

	let parsed = parse_blk(&file[offset..], file_type.is_slim(), nm)?;
	Ok(parsed)
}


pub(crate) fn test_parse_file(
	mut file: Vec<u8>,
	fd: Arc<BlkDecoder>,
	shared_name_map: Option<Arc<NameMap>>,
) -> Option<String> {
	let mut offset = 0;
	let file_type = FileType::from_byte(file[0]).ok()?;
	if file_type.is_zstd() {
		file = decode_zstd(file_type, &file, Some(fd.as_ref())).unwrap();
	} else {
		// uncompressed Slim and Fat files retain their initial magic bytes
		offset = 1;
	};

	Some(
		serde_json::to_string(
			&parse_blk(&file[offset..], file_type.is_slim(), shared_name_map).ok()?,
		)
			.unwrap(),
	)
}

pub fn make_strict_test() -> BlkField {
	BlkField::Struct(blk_str("root"), vec![
		BlkField::Value(blk_str("vec4f"), BlkType::Float4([1.25, 2.5, 5.0, 10.0])),
		BlkField::Value(blk_str("int"), BlkType::Int(42)),
		BlkField::Value(blk_str("long"), BlkType::Long(64)),
		BlkField::Struct(blk_str("alpha"), vec![
			BlkField::Value(blk_str("str"), BlkType::Str(blk_str("hello"))),
			BlkField::Value(blk_str("bool"), BlkType::Bool(true)),
			BlkField::Value(blk_str("color"), BlkType::Color { r: 3, g: 2, b: 1, a: 4 }),
			BlkField::Struct(blk_str("gamma"), vec![
				BlkField::Value(blk_str("vec2i"), BlkType::Int2([3, 4])),
				BlkField::Value(blk_str("vec2f"), BlkType::Float2([1.25, 2.5])),
				BlkField::Value(blk_str("transform"), BlkType::Float12(Box::new([1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 1.25, 2.5, 5.0]))),
			]),
		]),
		BlkField::Struct(blk_str("beta"), vec![
			BlkField::Value(blk_str("float"), BlkType::Float(1.25)),
			BlkField::Value(blk_str("vec2i"), BlkType::Int2([1, 2])),
			BlkField::Value(blk_str("vec3f"), BlkType::Float3([1.25, 2.5, 5.0])),
		]),
	])
}