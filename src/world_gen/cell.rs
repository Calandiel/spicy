use crate::constants::SQUARES_PER_CELL;

#[derive(Debug, Clone)]
pub struct OpenmwCell {
	elevations: [f32; SQUARES_PER_CELL * SQUARES_PER_CELL],
}

impl OpenmwCell {
	pub fn new() -> Self {
		OpenmwCell {
			elevations: [0.0; SQUARES_PER_CELL * SQUARES_PER_CELL],
		}
	}

	pub fn get_elevation(&self, square_within_cell: [u32; 2]) -> f32 {
		self.elevations
			[square_within_cell[0] as usize + square_within_cell[1] as usize * SQUARES_PER_CELL]
	}

	pub fn set_elevation(&mut self, square_within_cell: [u32; 2], value: f32) {
		self.elevations
			[square_within_cell[0] as usize + square_within_cell[1] as usize * SQUARES_PER_CELL] = value;
	}
}
