use clap::{command, Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "spicy")]
#[command(version = "0.0.1")]
#[command(about = "Game development tools for OpenMW", long_about = None)]
pub struct Args {
	#[command(subcommand)]
	pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
	#[command(about = "Creates a new spicy project")]
	New { path: String },
	#[command(about = "Clears all build and cache files")]
	Clear,
	#[command(about = "Runs the editor")]
	Edit,
	#[command(about = "Runs the game with OpenMW")]
	Run,
	#[command(about = "Compiles an out.omwgame file to run the game")]
	Compile,
	#[command(about = "Decompiles the out.omwgame in the build directory")]
	Decompile { input_path: Option<String> },
}
