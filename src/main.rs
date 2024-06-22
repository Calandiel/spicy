use anyhow::{anyhow, Ok};
use clap::Parser;
use serde_json::{from_str, Value};
use std::io::Write;
use std::{
	env,
	fs::{self, OpenOptions},
	path::PathBuf,
	process::{Command, Stdio},
};

fn get_record_types() -> Vec<String> {
	return vec![
		"Header".to_string(),
		"GameSetting".to_string(),
		"GlobalVariable".to_string(),
		"Class".to_string(),
		"Faction".to_string(),
		"Race".to_string(),
		"Sound".to_string(),
		"SoundGen".to_string(),
		"Skill".to_string(),
		"MagicEffect".to_string(),
		"Script".to_string(),
		"Region".to_string(),
		"StartScript".to_string(),
		"Birthsign".to_string(),
		"LandscapeTexture".to_string(),
		"Spell".to_string(),
		"Static".to_string(),
		"Door".to_string(),
		"MiscItem".to_string(),
		"Weapon".to_string(),
		"Container".to_string(),
		"Creature".to_string(),
		"Bodypart".to_string(),
		"Light".to_string(),
		"Enchanting".to_string(),
		"Npc".to_string(),
		"Armor".to_string(),
		"Clothing".to_string(),
		"RepairItem".to_string(),
		"Activator".to_string(),
		"Apparatus".to_string(),
		"Lockpick".to_string(),
		"Probe".to_string(),
		"Ingredient".to_string(),
		"Book".to_string(),
		"Alchemy".to_string(),
		"LeveledItem".to_string(),
		"LeveledCreature".to_string(),
		"Cell".to_string(),
		"Landscape".to_string(),
		"PathGrid".to_string(),
		"Dialogue".to_string(),
		"DialogueInfo".to_string(),
	];
}

/// A project management tool for OpenMW.
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// The action to perform
	#[arg(short, long)]
	action: Action,

	/// Path to the file to (de-)compile
	#[arg(short, long)]
	input_path: String,

	/// Path to save outputs at
	#[arg(short, long)]
	output_path: String,
}

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum Action {
	New,
	Clear,
	#[default]
	Compile,
	Decompile,
}

fn main() -> anyhow::Result<()> {
	println!("=== spicy (v0.0.1) ===");

	let args = Args::parse();
	println!("{:?}", args);

	match args.action {
		Action::New => todo!(),
		Action::Clear => todo!(),
		Action::Compile => todo!(),
		Action::Decompile => {
			decompile(args)?;
		}
	}

	return Ok(());
}

fn decompile(args: Args) -> anyhow::Result<()> {
	// Parse tes3conv path
	let mut tes3conv_path = env::current_dir().unwrap();
	println!("Current dir: {:?}", tes3conv_path);
	if cfg!(target_os = "windows") {
		tes3conv_path.push("common/tes3conv/windows/tes3conv.exe");
	} else if cfg!(target_os = "linux") {
		tes3conv_path.push("common/tes3conv/ubuntu/tes3conv");
	} else {
		panic!("Unsupported OS!");
	}
	if !tes3conv_path.exists() {
		panic!(
			"tes3conv file: {} does not exist.",
			tes3conv_path.to_string_lossy()
		)
	}

	// Parse paths
	let input_path = PathBuf::from(args.input_path);
	let mut output_path = env::current_dir().unwrap();
	output_path.push(args.output_path);
	println!("tes3conv path: {}", tes3conv_path.to_string_lossy());
	println!("Input path: {}", input_path.to_string_lossy());
	println!("Output path: {}", output_path.to_string_lossy());

	if !input_path.exists() {
		panic!(
			"Input file: {} does not exist.",
			input_path.to_string_lossy()
		)
	}

	// Remove the old file if it exists...
	if output_path.clone().exists() {
		fs::remove_file(output_path.clone()).unwrap();
	}

	println!("Running: {:?}\n", tes3conv_path);
	let output = Command::new(tes3conv_path)
		.arg(input_path.to_string_lossy().to_string())
		.arg(output_path.to_string_lossy().to_string())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.output()
		.expect("Failed to tes3conv");

	println!("{:?}", output);

	// Read back json for validation
	let json_data = fs::read_to_string(output_path.clone()).unwrap();
	let parsed_json: Value = from_str(&json_data).expect("Invalid JSON");
	validate_json(&parsed_json)?;

	// Once the json is validated, we can write it to individual directories
	// Create directories for record types
	let record_types = get_record_types();
	for record_type in record_types {
		let mut base_dir = env::current_dir().unwrap();
		base_dir.push("common/data");
		base_dir.push(record_type);
		if base_dir.exists() {
			fs::remove_dir_all(base_dir.clone())?;
		}
		fs::create_dir_all(base_dir)?;
	}

	// Create files for invividual record types and fill them with json
	println!("Creating record files...");
	let records = parsed_json.as_array().unwrap();
	let mut counter = 0;
	let mut all_file_names = vec![];
	for record in records {
		let record_type = record.get("type").unwrap().as_str().unwrap().to_string();
		let mut file_name = counter.to_string();
		if let Some(id) = record.get("id") {
			file_name = id.as_str().unwrap().to_string();
		} else if let Some(name) = record.get("name") {
			if name.as_str().unwrap().len() > 0 {
				file_name = name.as_str().unwrap().to_string();
			} else {
				counter += 1;
			}
		} else {
			counter += 1;
		}
		if all_file_names.contains(&file_name) {
			println!("Duplicate: {}", file_name);
			file_name.push_str("_");
			file_name.push_str(&counter.to_string());
			counter += 1;
		}
		all_file_names.push(file_name.clone());
		file_name.push_str(".json");

		let mut file_path = env::current_dir().unwrap();
		file_path.push("common/data");
		file_path.push(record_type);
		file_path.push(file_name);

		let file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(file_path.clone());
		match file {
			Err(_) => {
				println!();
				return Err(anyhow!("Failed to create a file: {:?}", file_path));
			}
			_ => {}
		}
		let stringified_record = serde_json::to_string_pretty(record)?;
		let data_to_write = stringified_record.as_bytes();
		file?.write(data_to_write)?;
	}

	// At the end, delete the file as we won't need it anymore.
	if output_path.clone().exists() {
		fs::remove_file(output_path.clone()).unwrap();
	}

	return Ok(());
}

fn validate_json(parsed_json: &Value) -> anyhow::Result<()> {
	if !parsed_json.is_array() {
		return Err(anyhow!("The outtermost json object must be an array!"));
	}

	let record_types = get_record_types();
	let records = parsed_json.as_array().unwrap();

	// Check for type presence and make sure all record types are known
	for record in records {
		let record_type = record.get("type");
		if let Some(record_type) = record_type {
			if !record_types.contains(&record_type.as_str().unwrap().to_string()) {
				return Err(anyhow!("Unknown record type: {}", record_type));
			}
		} else {
			return Err(anyhow!("A record has no type!"));
		}
	}

	return Ok(());
}
