use crate::constants::{LAND_RECORD_SCALER, SQUARES_PER_CELL, TODD_UNIT};

use super::cell::OpenmwCell;

#[derive(Debug, Clone)]
pub struct OpenmwWorld {
	pub overworld_cells: std::collections::HashMap<[i32; 2], OpenmwCell>,
}

impl OpenmwWorld {
	pub fn new() -> Self {
		OpenmwWorld {
			overworld_cells: std::collections::HashMap::default(),
		}
	}

	/*
	pub fn convert_to_json(&self) -> anyhow::Result<()> {
		if PathBuf::from("common/data/Cell").exists() {
			fs::remove_dir_all("common/data/Cell")?;
		}

		for (coords, cell) in &self.overworld_cells {
			let mut cell_json = serde_json::json!(r#"{ "type": "Cell" }"#);
		}

		Ok(())
	}
	*/

	/*
	// Value should be in METERS, not tuds
	pub fn set_elevation(&mut self, square: [i32; 2], value: f32) {
		println!("{}", value);
		// get the cell
		let square_within_cell = get_coords_within_a_cell(square);
		let cell = get_cell(&mut self.overworld_cells, square);
		cell.set_elevation(square_within_cell, value);
	}
	*/

	// Value should be in heightmap adjusted tuds
	pub fn set_elevation_canonical(&mut self, square: [i32; 2], value: f32) {
		println!("{}", value);
		// get the cell
		let square_within_cell = get_coords_within_a_cell(square);
		let cell = get_cell(&mut self.overworld_cells, square);
		cell.set_elevation(
			square_within_cell,
			value as f32 * LAND_RECORD_SCALER / TODD_UNIT,
		);
	}

	// Value is in METERS, not tuds
	pub fn get_elevation(&mut self, square: [i32; 2]) -> f32 {
		// get the cell
		let cell = get_cell(&mut self.overworld_cells, square);
		let square_within_cell = get_coords_within_a_cell(square);
		cell.get_elevation(square_within_cell)
	}

	// Returns elevation in heightmap adjusted tuds
	pub fn get_elevation_canonical(&mut self, square: [i32; 2]) -> i32 {
		let ele = self.get_elevation(square);
		(ele / LAND_RECORD_SCALER * TODD_UNIT) as i32
	}
}

fn get_cell_grid_coords(square: [i32; 2]) -> [i32; 2] {
	[
		square[0].div_euclid(SQUARES_PER_CELL as i32),
		square[1].div_euclid(SQUARES_PER_CELL as i32),
	]
}

fn get_coords_within_a_cell(square: [i32; 2]) -> [u32; 2] {
	let cell = get_cell_grid_coords(square);
	[
		(square[0] - cell[0] * SQUARES_PER_CELL as i32) as u32,
		(square[1] - cell[1] * SQUARES_PER_CELL as i32) as u32,
	]
}

fn get_cell(
	overworld_cells: &mut std::collections::HashMap<[i32; 2], OpenmwCell>,
	square: [i32; 2],
) -> &mut OpenmwCell {
	let cell = get_cell_grid_coords(square);
	if !overworld_cells.contains_key(&cell) {
		overworld_cells.insert(cell, OpenmwCell::new());
	}
	overworld_cells.get_mut(&cell).unwrap()
}
