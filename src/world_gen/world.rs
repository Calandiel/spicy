use std::{fs, path::PathBuf};

use crate::constants::SQUARES_PER_CELL;

use super::cell::OpenmwCell;

#[derive(Debug, Clone)]
pub struct OpenmwWorld {
	pub overworld_cells: std::collections::HashMap<[u32; 2], OpenmwCell>,
}

impl OpenmwWorld {
	pub fn new() -> Self {
		OpenmwWorld {
			overworld_cells: std::collections::HashMap::default(),
		}
	}

	pub fn convert_to_json(&self) -> anyhow::Result<()> {
		if PathBuf::from("common/data/Cell").exists() {
			fs::remove_dir_all("common/data/Cell")?;
		}

		for (coords, cell) in &self.overworld_cells {
			let mut cell_json = serde_json::json!(r#"{ "type": "Cell" }"#);
		}

		Ok(())
	}

	pub fn set_elevation(&mut self, square: [u32; 2], value: f32) {
		// get the cell
		let cell = get_cell(&mut self.overworld_cells, square);
		let square_within_cell = get_coords_within_a_cell(square);
		cell.set_elevation(square_within_cell, value);
	}
}

fn get_cell_grid_coords(square: [u32; 2]) -> [u32; 2] {
	[
		square[0] / SQUARES_PER_CELL as u32,
		square[1] / SQUARES_PER_CELL as u32,
	]
}

fn get_coords_within_a_cell(square: [u32; 2]) -> [u32; 2] {
	let cell = get_cell_grid_coords(square);
	[
		square[0] - cell[0] * SQUARES_PER_CELL as u32,
		square[1] - cell[1] * SQUARES_PER_CELL as u32,
	]
}

fn get_cell(
	overworld_cells: &mut std::collections::HashMap<[u32; 2], OpenmwCell>,
	square: [u32; 2],
) -> &mut OpenmwCell {
	let cell = get_cell_grid_coords(square);
	if !overworld_cells.contains_key(&cell) {
		overworld_cells.insert(cell, OpenmwCell::new());
	}
	overworld_cells.get_mut(&cell).unwrap()
}
