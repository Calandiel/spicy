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
	input_path: Option<String>,
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

	let _ = ensure_common_exists()?;

	match args.action {
		Action::New => {
			let _json = r###"
{
  "author": "Author",
  "description": "Data",
  "file_type": "Esm",
  "masters": [],
  "type": "Header",
  "version": 1.3
}
"###;
			//  "num_objects": 0,
			//  "flags": "",
		}
		Action::Clear => {
			clear()?;
		}
		Action::Compile => {
			compile()?;
		}
		Action::Decompile => {
			decompile(args)?;
		}
	}

	Ok(())
}

fn get_tes3conv_path() -> PathBuf {
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

	tes3conv_path
}

fn process_directory(input_path: PathBuf, outputs: &mut Vec<PathBuf>) -> anyhow::Result<()> {
	let directories = fs::read_dir(input_path).unwrap();
	for directory in directories {
		let directory_entry = directory?;
		let directory_path = directory_entry.path();

		if directory_path.is_dir() {
			process_directory(directory_path.clone(), outputs)?;
		}

		if directory_path.is_file() {
			if let Some(ext) = directory_path.extension() {
				if ext == "json" {
					outputs.push(directory_path.clone());
					println!("{:?}", directory_path);
				}
			}
		}
	}
	Ok(())
}

fn ensure_common_exists() -> anyhow::Result<()> {
	if !PathBuf::from("common/cache").exists()
		|| !PathBuf::from("common/build").exists()
		|| !PathBuf::from("common/data").exists()
	{
		clear()?;
	}

	let _ =ensure_tes3conv_exists();

	Ok(())
}

fn ensure_tes3conv_exists() -> anyhow::Result<()> {
		if !PathBuf::from("common/tes3conv").exists() {
		// Get raw data
		let tes3conv_windows = include_bytes!("tes3conv/windows/tes3conv.exe");
		let tes3conv_linux = include_bytes!("tes3conv/ubuntu/tes3conv");

		// Create dirs
		let mut path = env::current_dir().unwrap();
		path.push("common/tes3conv/windows");
		fs::create_dir_all(path.clone())?;

		let mut path = env::current_dir().unwrap();
		path.push("common/tes3conv/ubuntu");
		fs::create_dir_all(path.clone())?;

		// Create and write files
		let mut path = env::current_dir().unwrap();
		path.push("common/tes3conv/windows/tes3conv.exe");
		let mut file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(path.clone())?;
		file.write(tes3conv_windows)?;

		let mut path = env::current_dir().unwrap();
		path.push("common/tes3conv/ubuntu/tes3conv");
		let mut file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(path.clone())?;
		file.write(tes3conv_linux)?;
	}

	Ok(())
}

fn clear() -> anyhow::Result<()> {
	if PathBuf::from("common/cache").exists() {
		fs::remove_dir_all("common/cache")?;
	}
	if PathBuf::from("common/build").exists() {
		fs::remove_dir_all("common/build")?;
	}
	if PathBuf::from("common/data").exists() {
		fs::remove_dir_all("common/data")?;
	}
	if PathBuf::from("common/tes3conv").exists() {
		fs::remove_dir_all("common/tes3conv")?;
	}

	let create_gitkeep = |subpath: &str, file: &str| -> anyhow::Result<()> {
		let mut path = env::current_dir().unwrap();
		path.push(subpath);

		fs::create_dir_all(path.clone())?;

		path.push(file);

		let _ = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(path.clone())?;

		Ok(())
	};

	let _ = create_gitkeep("common/cache", ".gitkeep")?;
	let _ = create_gitkeep("common/build", ".gitkeep")?;
	let _ = create_gitkeep("common/data", ".gitkeep")?;
	let _ = ensure_tes3conv_exists()?;

	Ok(())
}

fn compile() -> anyhow::Result<()> {
	let tes3conv_path = get_tes3conv_path();

	// Parse paths
	let mut input_path = env::current_dir().unwrap();
	input_path.push("common/data");
	let mut output_path = env::current_dir().unwrap();
	output_path.push("common/build/out.esm");
	let mut temporary_json_path = env::current_dir().unwrap();
	temporary_json_path.push("common/cache/temp.json");
	println!("tes3conv path: {}", tes3conv_path.to_string_lossy());
	println!("Output path: {}", output_path.to_string_lossy());

	// Remove the old file if it exists...
	if output_path.clone().exists() {
		fs::remove_file(output_path.clone()).unwrap();
	}

	let mut files = vec![];
	process_directory(input_path, &mut files)?;

	let mut json = r"[".to_string();
	for (index, file) in files.iter().enumerate() {
		let json_data = fs::read_to_string(file).unwrap();
		json.push_str(&json_data);
		if index != files.len() - 1 {
			json.push_str(",");
		}
	}
	json.push_str("]");

	println!("Saving final json...");
	let mut file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(temporary_json_path.clone())?;
	file.write(json.as_bytes())?;

	println!("Running: {:?}\n", tes3conv_path);
	let output = Command::new(tes3conv_path)
		.arg(temporary_json_path.to_string_lossy().to_string())
		.arg(output_path.to_string_lossy().to_string())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.output()
		.expect("Failed to tes3conv");

	println!("{:?}", output);

	Ok(())
}

fn decompile(args: Args) -> anyhow::Result<()> {
	let tes3conv_path = get_tes3conv_path();

	// Parse paths
	let mut input_path = PathBuf::from(
		args.input_path
			.expect("Missing input path for the decompilation step"),
	);
	let mut output_path = env::current_dir().unwrap();
	output_path.push("common/cache/temp.json");
	println!("tes3conv path: {}", tes3conv_path.to_string_lossy());
	println!("Input path: {}", input_path.to_string_lossy());
	println!("Output path: {}", output_path.to_string_lossy());

	if input_path.extension().unwrap_or_default() == "omwgame"
		|| input_path.extension().unwrap_or_default() == "omwaddon"
	{
		println!("Working with openmw... Converting extension...");
		let data = fs::read(input_path.clone())?;

		let mut new_input_path = env::current_dir().unwrap();
		new_input_path.push("common/cache/temp.esm");

		let mut file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(new_input_path.clone())?;
		file.write(&data)?;

		input_path = new_input_path;
	}

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

	Ok(())
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

	Ok(())
}
