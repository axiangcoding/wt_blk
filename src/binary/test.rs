#[cfg(test)]
mod test {
	use crate::binary::blk_type::BlkType;
	use crate::binary::leb128::uleb128;

	#[test]
	fn fat_blk() {
		let file = include_bytes!("../../samples/section_fat.blk");
		let mut ptr = 0;

		let file_type = file[0];
		ptr += 1;

		let (offset, names_count) = uleb128(&file[ptr..]).unwrap();
		ptr += offset;

		let (offset, names_data_size) = uleb128(&file[ptr..]).unwrap();
		ptr += offset;

		let mut names = vec![];

		{
			let mut buff = vec![];
			for idx in 0..names_data_size {
				let char = file[ptr + idx];
				if char == 0 {
					names.push(String::from_utf8(buff.clone()).unwrap());
					buff.clear();
				} else {
					buff.push(char);
				}
			}
			ptr += names_data_size;
		}

		let (offset, blocks_count) = uleb128(&file[ptr..]).unwrap();
		ptr += offset;

		let (offset, params_count) = uleb128(&file[ptr..]).unwrap();
		ptr += offset;

		let (offset, params_data_size) = uleb128(&file[ptr..]).unwrap();
		ptr += offset;

		let params_data = &file[ptr..(ptr + params_data_size)];
		ptr += params_data_size;

		let params_info = &file[ptr..(ptr + params_count * 8)];
		ptr += params_info.len();

		let block_info = &file[ptr..];
		drop(ptr);

		let dbg_hex = |x: &[u8]| x.iter().map(|item| format!("{:X}", item)).collect::<Vec<String>>().join(" ");

		let mut results = vec![];
		for chunk in params_info.chunks(8) {
			let name_id_raw = &chunk[0..3];
			let name_id = u32::from_le_bytes([
				name_id_raw[0],
				name_id_raw[1],
				name_id_raw[2],
				0
			]);
			let type_id = chunk[3];
			let data = &chunk[4..];
			let name = &names[name_id as usize];

			let parsed = BlkType::from_raw_param_info(type_id, data, params_data).unwrap();
			results.push((name_id as usize, name, parsed));
		}
		println!("{:?}", results);
	}
}