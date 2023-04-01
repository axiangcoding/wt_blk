use std::ops::Range;
use std::rc::Rc;

use crate::binary::blk_structure::BlkField;
use crate::binary::blk_type::BlkString;

#[derive(Debug, Clone)]
pub struct FlatBlock {
	pub name: BlkString,
	pub fields: Vec<BlkField>,
	pub blocks: usize,
	pub offset: usize,
}

impl FlatBlock {
	fn location_range(&self) -> Range<usize> {
		self.offset..(self.offset + self.blocks)
	}
}

impl BlkField {
	pub fn from_flat_blocks(flat_blocks: Vec<FlatBlock>) -> Self {
		let cloned = flat_blocks[0].clone();
		Self::from_flat_blocks_with_parent(&flat_blocks, cloned)
	}

	fn from_flat_blocks_with_parent(flat_blocks: &Vec<FlatBlock>, parent: FlatBlock) -> Self {
		let mut block = BlkField::Struct(parent.name.clone(), parent.fields.clone());

		for flat_block in &flat_blocks[parent.location_range()] {
			block.insert_field(Self::from_flat_blocks_with_parent(flat_blocks, flat_block.clone())).unwrap();
		}

		block
	}
}