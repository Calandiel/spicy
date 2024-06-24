use clap::Parser;

/// A project management tool for OpenMW.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
	/// The action to perform
	#[arg(short, long)]
	pub action: Action,

	/// Path to the file to (de-)compile
	#[arg(short, long)]
	pub input_path: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
pub enum Action {
	New,
	Clear,
	#[default]
	Compile,
	Decompile,
}
