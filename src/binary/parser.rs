use std::borrow::Cow;
use std::ops::DerefMut;
use std::rc::Rc;
use std::time::Instant;
use tracing::{error, warn};
use crate::binary::blk_block_hierarchy::FlatBlock;
use crate::binary::blk_structure::BlkField;
use crate::binary::blk_type::{BlkString, BlkType};
use crate::binary::error::ParseError;
use crate::binary::error::ParseError::{BadBlkValue, ResidualBlockBuffer};
use crate::binary::file::FileType;
use crate::binary::leb128::uleb128;
use crate::binary::nm_file::NameMap;

pub fn parse_blk(file: &[u8], is_slim: bool, shared_name_map: Rc<NameMap>) -> Result<BlkField, ParseError> {
	let mut ptr = 0;

	// Globally increments ptr and returns next uleb integer from file
	let next_uleb = |ptr: &mut usize| {
		// Using ? inside of closures is not supported yet, so we need to use this match
		match uleb128(&file[*ptr..]) {
			Ok((offset, int)) => {
				*ptr += offset;
				Ok(int)
			}
			Err(e) => { Err(e) }
		}
	};

	// Returns slice offset from file, incrementing the ptr by offset
	let idx_file_offset = |ptr: &mut usize, offset: usize| {
		let res = file.get(*ptr..(*ptr + offset)).ok_or(ParseError::DataRegionBoundsExceeded(*ptr..(*ptr + offset)));
		*ptr += offset;
		res
	};


	let names_count = next_uleb(&mut ptr)?;

	let names = if is_slim { // TODO Figure out if names_count dictates the existence of a name map or if it may be 0 without requiring a name map
		shared_name_map.parsed.clone()
	} else {
		let names_data_size = next_uleb(&mut ptr)?;

		let names = NameMap::parse_name_section(idx_file_offset(&mut ptr, names_data_size)?);
		if names_count != names.len() {
			error!("Name count mismatch, expected {names_count}, but found a len of {}. This might mean something is wrong.", names.len());
		}
		Rc::new(names)
	};

	let blocks_count = next_uleb(&mut ptr)?;

	let params_count = next_uleb(&mut ptr)?;

	let params_data_size = next_uleb(&mut ptr)?;

	let params_data = idx_file_offset(&mut ptr, params_data_size)?;

	let params_info= idx_file_offset(&mut ptr, params_count * 8)?;

	let block_info = &file.get(ptr..).ok_or(ResidualBlockBuffer)?;

	let ptr = (); // Shadowing ptr causes it to become unusable, especially on accident

	let mut results: Vec<(usize, BlkField)> = Vec::with_capacity(params_info.len() / 8);

	let chunks = params_info.chunks_exact(8);
	if chunks.remainder().len() != 0 { error!("Params info chunks did not align to 8 bytes") } // TODO: Decide whether or not this constitutes a hard crash
	for chunk in chunks {
		let name_id_raw = &chunk[0..3];
		let name_id = u32::from_le_bytes([
			name_id_raw[0],
			name_id_raw[1],
			name_id_raw[2],
			0
		]);
		let type_id = chunk[3];
		let data = &chunk[4..];
		let name = names[name_id as usize].clone();


		// TODO: Validate wether or not slim files store only strings in the name map
		let parsed = if is_slim && type_id == 0x01 {
			BlkType::from_raw_param_info(type_id, data, shared_name_map.binary.clone(), shared_name_map.parsed.clone()).ok_or(BadBlkValue)?
		} else {
			BlkType::from_raw_param_info(type_id, data, Rc::new(params_data.to_owned()), names.clone()).ok_or(BadBlkValue)?
		};

		let field = BlkField::Value(name, parsed);
		results.push((name_id as usize, field));
	}

	let mut blocks = Vec::with_capacity(blocks_count);
	{
		let block_id_to_name = |id| {
			if id == 0 {
				Rc::new("root".to_owned())
			} else {
				(&names)[(id - 1) as usize].clone()
			}
		};

		let mut ptr = 0;
		for _ in 0..blocks_count {
			let (offset, name_id) = uleb128(&block_info[ptr..]).unwrap();
			ptr += offset;

			let (offset, param_count) = uleb128(&block_info[ptr..]).unwrap();
			ptr += offset;

			let (offset, blocks_count) = uleb128(&block_info[ptr..]).unwrap();
			ptr += offset;

			let first_block_id = if blocks_count > 0 {
				let (offset, first_block_id) = uleb128(&block_info[ptr..]).unwrap();
				ptr += offset;
				Some(first_block_id)
			} else {
				None
			};

			blocks.push((block_id_to_name(name_id), param_count, blocks_count, first_block_id));
			// Name of the block
			// Amount of non-block fields
			// Amount of child-blocks
			// If it has child-blocks, starting index of said block
		}
	}

	// Create a flat hierarchy of all blocks including their non-block fields
	// This ensures all values are actually assigned
	// After this, the hierarchy will be assigned depth depending on the block-map
	let mut flat_map: Vec<FlatBlock> = Vec::with_capacity(blocks_count);
	let mut ptr = 0;
	for (name, field_count, blocks, offset) in blocks {
		let mut field = FlatBlock {
			name,
			fields: Vec::with_capacity(field_count),
			blocks,
			offset: offset.unwrap_or(0),
		};
		for i in (ptr)..(ptr + field_count) {
			field.fields.push(results[i].1.clone());
		}
		ptr += field_count;
		flat_map.push(field);
	}

	let out = BlkField::from_flat_blocks(flat_map);
	Ok(out)
}