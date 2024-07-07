use crate::{
	args::{Args, Commands},
	constants::{ORIGINAL_FILE_PATH_JSON_ATTR, SQUARES_PER_CELL},
	utils::create_subdirectory,
};
use anyhow::{anyhow, Ok};
use base64::Engine;
use clap::Parser;
use colored::*;
use dae::get_target_path;
use records::get_record_types;
use serde_json::{from_str, Value};
use std::{
	env,
	fs::{self, OpenOptions},
	io::Write,
	path::PathBuf,
	process::{Command, Stdio},
};
use utils::{create_bin_file, create_text_file};
use world_gen::world::OpenmwWorld;

mod args;
mod constants;
mod dae;
mod record;
mod records;
mod utils;
mod world_gen;

fn main() -> anyhow::Result<()> {
	if let Err(e) = main_body() {
		println!("{}", "Failed".red());
		return Err(e);
	}

	println!("{}", "Finished".green());
	Ok(())
}

fn main_body() -> anyhow::Result<()> {
	let args = Args::parse();

	match args.command {
		Commands::New { path } => {
			new(path)?;
		}
		Commands::Run => {
			run()?;
		}
		Commands::Clear => {
			ensure_common_exists()?;
			clear()?;
		}
		Commands::Compile => {
			ensure_common_exists()?;
			compile()?;
		}
		Commands::Decompile { input_path } => {
			ensure_common_exists()?;
			decompile(input_path)?;
		}
	}

	Ok(())
}

fn check_for_spicy_toml() -> anyhow::Result<()> {
	let mut path = env::current_dir().unwrap();
	path.push("spicy.toml");
	if path.exists() {
		Ok(())
	} else {
		Err(anyhow!(
			"could not find `spicy.toml` in `{}`",
			path.to_string_lossy()
		))
	}
}

fn get_tes3conv_path() -> PathBuf {
	// Parse tes3conv path
	let mut tes3conv_path = env::current_dir().unwrap();
	println!("Current dir: {:.?}", tes3conv_path);
	if cfg!(target_os = "windows") {
		tes3conv_path.push("cache/tes3conv/windows/tes3conv.exe");
	} else if cfg!(target_os = "linux") {
		tes3conv_path.push("cache/tes3conv/linux/tes3conv");
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

fn process_directory(mut input_path: PathBuf, outputs: &mut Vec<PathBuf>) -> anyhow::Result<()> {
	if cfg!(target_os = "windows") {
		let new_input_path = input_path.clone().to_string_lossy().replace("/", "\\");
		input_path = new_input_path.into();
	}
	let directories = fs::read_dir(input_path).unwrap();
	for directory in directories {
		let directory_entry = directory.unwrap();
		let directory_path = directory_entry.path();

		if directory_path.is_dir() {
			process_directory(directory_path.clone(), outputs).unwrap();
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
	if !PathBuf::from("cache").exists()
		|| !PathBuf::from("build").exists()
		|| !PathBuf::from("common/data").exists()
	{
		clear()?;
	}

	let _ = ensure_tes3conv_exists();

	Ok(())
}

// fn ensure_openmw_exists() -> anyhow::Result<()> {

// }

fn ensure_tes3conv_exists() -> anyhow::Result<()> {
	if !PathBuf::from("cache/tes3conv").exists() {
		// Get raw data
		let tes3conv_windows = include_bytes!("tes3conv/windows/tes3conv.exe");
		let tes3conv_linux = include_bytes!("tes3conv/linux/tes3conv");

		// Create dirs
		let mut path = env::current_dir().unwrap();
		path.push("cache/tes3conv/windows");
		fs::create_dir_all(path.clone()).unwrap();

		let mut path = env::current_dir().unwrap();
		path.push("cache/tes3conv/linux");
		fs::create_dir_all(path.clone()).unwrap();

		// Create and write files
		create_bin_file("cache/tes3conv/windows", "tes3conv.exe", tes3conv_windows).unwrap();
		create_bin_file("cache/tes3conv/linux", "tes3conv", tes3conv_linux).unwrap();

		set_exe_permissions("cache/tes3conv/linux/tes3conv").unwrap();
	}

	Ok(())
}

#[cfg(target_os = "linux")]
fn set_exe_permissions(linux_path: &str) -> anyhow::Result<()> {
	use std::os::unix::fs::PermissionsExt;
	let metadata = std::fs::metadata(linux_path).unwrap();
	let mut permissions = metadata.permissions();
	permissions.set_mode(0o775);
	fs::set_permissions(linux_path, permissions).unwrap();

	Ok(())
}

#[cfg(target_os = "windows")]
fn set_exe_permissions(linux_path: &str) -> anyhow::Result<()> {
	println!(
		"not on linux so we're not setting permissions on {:?}",
		linux_path
	);
	Ok(())
}

fn run() -> anyhow::Result<()> {
	compile()?;
	Ok(())
}

fn new(relative_path: String) -> anyhow::Result<()> {
	let mut base_path = env::current_dir().unwrap();
	base_path.push(relative_path.clone());
	if base_path.exists() {
		return Err(anyhow!(
			"directory `{}` already exists",
			base_path.to_string_lossy()
		));
	}
	fs::create_dir_all(base_path.clone())?;
	let mut spicy_toml = base_path.clone();
	spicy_toml.push("spicy.toml");
	fs::write(spicy_toml, "# The project was created by spicy")?;

	create_record_dirs(Some(relative_path.clone()))?;

	let json = r###"{
  "author": "Author",
  "description": "Spicy plugin for OpenMW",
  "file_type": "Esm",
  "flags": "",
  "masters": [],
  "type": "Header",
  "version": 1.3
}
"###;

	create_text_file(
		format!("{}/common/data/Header", relative_path).as_str(),
		"header.json",
		json,
	)
	.unwrap();

	Ok(())
}

fn clear() -> anyhow::Result<()> {
	check_for_spicy_toml()?;
	if PathBuf::from("cache").exists() {
		fs::remove_dir_all("cache").unwrap();
	}
	if PathBuf::from("build").exists() {
		fs::remove_dir_all("build").unwrap();
	}

	Ok(())
}

fn compile() -> anyhow::Result<()> {
	check_for_spicy_toml()?;
	create_subdirectory("cache").unwrap();
	create_subdirectory("build").unwrap();
	ensure_tes3conv_exists().unwrap();
	dae::compile_assets()?;

	let tes3conv_path = get_tes3conv_path();

	// Parse paths
	let mut input_path = env::current_dir().unwrap();
	input_path.push("common/data");
	let mut output_path = env::current_dir().unwrap();
	output_path.push("build/out.esm");
	let mut temporary_json_path = env::current_dir().unwrap();
	temporary_json_path.push("cache/temp.json");
	println!("tes3conv path: {}", tes3conv_path.to_string_lossy());
	println!("Output path: {}", output_path.to_string_lossy());
	let mut final_path = output_path.clone();
	final_path.set_extension("omwgame");

	if final_path.exists() {
		fs::remove_file(final_path.clone()).unwrap();
	}
	// Remove the old file if it exists...
	if output_path.clone().exists() {
		fs::remove_file(output_path.clone()).unwrap();
	}

	let mut files = vec![];
	process_directory(input_path, &mut files).unwrap();

	let mut cell_reference_counter = 0;
	let mut dialogue_info_id_counter = 0;
	let mut json = r"[".to_string();
	let mut parsed_jsons = vec![];
	for file in &files {
		println!("Parsing: {:?}", file);
		let json_data = fs::read_to_string(file).unwrap();
		let mut parsed_json: Value = from_str(&json_data).expect("Invalid JSON");
		parsed_json.as_object_mut().unwrap().insert(
			ORIGINAL_FILE_PATH_JSON_ATTR.to_string(),
			file.to_string_lossy().into(),
		);

		// Validate individual records...
		fill_in_single_record(&mut parsed_json, files.len(), &mut cell_reference_counter).unwrap();
		validate_single_record(&parsed_json).unwrap();

		parsed_jsons.push(parsed_json);
	}
	// Validate all recorda at once
	validate_records_together(&mut parsed_jsons).unwrap();

	// Write the final value
	for (index, parsed_json) in parsed_jsons.iter_mut().enumerate() {
		let mut infos = vec![];
		if read_string_from_record(&parsed_json, "type").unwrap() == "Dialogue" {
			infos = parsed_json
				.get("dialogue_infos")
				.unwrap()
				.as_array()
				.unwrap()
				.clone();
			parsed_json.as_object_mut().unwrap().remove("dialogue_info");
		}
		json.push_str(&parsed_json.to_string());

		// Preprocess infos
		for idx in 0..infos.len() {
			infos[idx].as_object_mut().unwrap().insert(
				"id".to_string(),
				dialogue_info_id_counter.to_string().into(),
			);
			if idx == 0 {
				infos[idx]
					.as_object_mut()
					.unwrap()
					.insert("prev_id".to_string(), "".into());
			} else {
				infos[idx].as_object_mut().unwrap().insert(
					"prev_id".to_string(),
					(dialogue_info_id_counter - 1).to_string().into(),
				);
			}
			if idx == infos.len() - 1 {
				infos[idx]
					.as_object_mut()
					.unwrap()
					.insert("next_id".to_string(), "".into());
			} else {
				infos[idx].as_object_mut().unwrap().insert(
					"next_id".to_string(),
					(dialogue_info_id_counter + 1).to_string().into(),
				);
			}
			dialogue_info_id_counter += 1;
		}

		if read_string_from_record(&parsed_json, "type").unwrap() == "Dialogue" {
			for info in &mut infos {
				fill_in_single_record(info, 0, &mut cell_reference_counter).unwrap();
				validate_single_record(info).unwrap();

				json.push_str(",\n");
				json.push_str(&info.to_string());
			}
		}
		if index != files.len() - 1 {
			json.push_str(",\n");
		}
	}
	json.push(']');

	validate_json(&from_str(&json).unwrap()).unwrap();

	println!("Saving final json...");
	let mut file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(temporary_json_path.clone())
		.unwrap();
	file.write_all(json.as_bytes()).unwrap();

	println!("Running: {:?}\n", tes3conv_path);
	let output = Command::new(tes3conv_path)
		.arg(temporary_json_path.to_string_lossy().to_string())
		.arg(output_path.to_string_lossy().to_string())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.output()
		.expect("Failed to tes3conv");

	let converted = fs::read(output_path.clone()).unwrap();
	fs::write(final_path, converted).unwrap();
	fs::remove_file(output_path).unwrap(); // remove after copying

	println!("{:?}", output);

	Ok(())
}

fn decompile(input_path: Option<String>) -> anyhow::Result<()> {
	check_for_spicy_toml()?;
	let tes3conv_path = get_tes3conv_path();

	// Parse paths
	let mut input_path = PathBuf::from(input_path.unwrap_or("build/out.omwgame".to_string()));
	let mut output_path = env::current_dir().unwrap();
	output_path.push("cache/temp.json");
	println!("tes3conv path: {}", tes3conv_path.to_string_lossy());
	println!("Input path: {}", input_path.to_string_lossy());
	println!("Output path: {}", output_path.to_string_lossy());

	if input_path.extension().unwrap_or_default() == "omwgame"
		|| input_path.extension().unwrap_or_default() == "omwaddon"
	{
		println!("Working with openmw... Converting extension...");
		let data = fs::read(input_path.clone()).unwrap();

		let mut new_input_path = env::current_dir().unwrap();
		new_input_path.push("cache/temp.esm");

		let mut file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(new_input_path.clone())
			.unwrap();
		file.write_all(&data).unwrap();

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

	// Read back json for validation // .unwrap().unwrap().unwrap() why would we validate on DECOMPILEs.unwrap()
	let json_data = fs::read_to_string(output_path.clone()).unwrap();
	let parsed_json: Value = from_str(&json_data).expect("Invalid JSON");
	// validate_json(&parsed_json).unwrap();

	create_record_dirs(None).unwrap();

	// Create files for invividual record types and fill them with json
	println!("Creating record files...");
	let mut records = parsed_json.as_array().unwrap().clone();

	// Update dialogue infos on dialogues
	let mut last_dialogue = 0;
	for idx in 0..records.len() {
		let record_type = records[idx]
			.get("type")
			.unwrap()
			.as_str()
			.unwrap()
			.to_string();
		if record_type == "Cell" {
			// For cells, we want to remove the mast_index and ref_index because we strictly want to create new games, not mods!
			// We can later make it an optional functionality instead, though.
			let refs = records[idx]
				.get_mut("references")
				.unwrap()
				.as_array_mut()
				.unwrap();
			for re in refs {
				re.as_object_mut().unwrap().remove("mast_index");
				re.as_object_mut().unwrap().remove("refr_index");
			}
		}
		if record_type == "Dialogue" {
			let infos: Vec<Value> = vec![];
			records[idx]
				.as_object_mut()
				.unwrap()
				.insert("dialogue_infos".to_string(), infos.into());
			last_dialogue = idx;
		}
		if record_type == "DialogueInfo" {
			let mut rec = records[idx].clone();
			rec.as_object_mut().unwrap().remove("id").unwrap();
			rec.as_object_mut().unwrap().remove("next_id").unwrap();
			rec.as_object_mut().unwrap().remove("prev_id").unwrap();

			records[last_dialogue]
				.get_mut("dialogue_infos")
				.unwrap()
				.as_array_mut()
				.unwrap()
				.push(rec);
		}
		// if record_type == "Landscape" {
		// let data_in_json = records
		// .get_mut(idx)
		// .unwrap()
		// .get_mut("vertex_heights")
		// .unwrap()
		// .get_mut("data")
		// .unwrap();
		// let mut heights = vec![];
		// use base64::prelude::*;
		// let decoded = BASE64_STANDARD.decode(data_in_json.as_str().unwrap()).unwrap();
		// for chunk in &decoded[4..] {
		// heights.push(*chunk as i8);
		// }
		// *data_in_json = heights.into();
		// }
	}

	let mut counter = 0;
	let mut all_file_names = vec![];
	for record in &records {
		let record_type = record.get("type").unwrap().as_str().unwrap().to_string();

		if record_type == "DialogueInfo" {
			continue; // skip dialogue infos
		}
		let mut file_name = counter.to_string();
		if let Some(id) = record.get("id") {
			file_name = id.as_str().unwrap().to_string();
		} else if let Some(name) = record.get("name") {
			if !name.as_str().unwrap().is_empty() {
				file_name = name.as_str().unwrap().to_string();
			} else {
				counter += 1;
			}
		} else {
			counter += 1;
		}
		if all_file_names.contains(&file_name) {
			println!("Duplicate: {}", file_name);
			file_name.push('_');
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
		if file.is_err() {
			println!();
			return Err(anyhow!("Failed to create a file: {:?}", file_path));
		}
		let stringified_record = serde_json::to_string_pretty(record).unwrap();
		let data_to_write = stringified_record.as_bytes();
		file.unwrap().write_all(data_to_write).unwrap();
	}

	// At the end, delete the file as we won't need it anymore.
	if output_path.clone().exists() {
		fs::remove_file(output_path.clone()).unwrap();
	}

	Ok(())
}

fn create_record_dirs(base_path: Option<String>) -> anyhow::Result<()> {
	// Once the json is validated, we can write it to individual directories
	// Create directories for record types
	let record_types = get_record_types();
	for record_type in record_types {
		let mut base_dir = env::current_dir().unwrap();
		if let Some(base_path) = base_path.as_ref() {
			base_dir.push(base_path);
		}
		base_dir.push("common/data");
		base_dir.push(record_type);
		if base_dir.exists() {
			fs::remove_dir_all(base_dir.clone())?;
		}
		fs::create_dir_all(base_dir)?;
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
				if record_type.as_str().unwrap() == "DialogueInfo" {
					// Dialogue infos need to be handled differently...
				} else {
					return Err(anyhow!("Unknown record type: {}", record_type));
				}
			}
		} else {
			return Err(anyhow!("A record has no type!"));
		}

		validate_single_record(record).unwrap();
	}

	let mut all_ids = std::collections::HashSet::new();

	// Check for id correctness and duplicates
	for record in records {
		let record_type = read_string_from_record(record, "type").unwrap();
		if let anyhow::Result::Ok(id) = read_string_from_record(record, "id") {
			println!("ID: |{}| - {}", id, record_type);

			if id.len() == 0 {
				return Err(anyhow!("A record has an id of length 0!"));
			}
			let first = id.chars().next().unwrap();
			if !first.is_alphabetic() && record_type != "DialogueInfo" {
				return Err(anyhow!(
					"Id {} doesn't start with an alphabetic character!",
					id
				));
			}

			if !all_ids.insert(id.clone()) {
				return Err(anyhow!("Duplicate id: {}", id));
			}
		}
	}

	Ok(())
}

fn read_string_from_record(record: &Value, attribute: &str) -> anyhow::Result<String> {
	let attribute_value = record.get(attribute);

	if let Some(attribute_value) = attribute_value {
		if let Some(attribute_value) = attribute_value.as_str() {
			return Ok(attribute_value.to_string());
		}
	}

	Err(anyhow!("Record has no string attribute {}", attribute))
}

// The first argument of the closure is in heightmap adjusted tuds
fn do_for_all_landscapes<F>(records: &mut Vec<Value>, mut closure: F) -> anyhow::Result<()>
where
	F: FnMut(&mut f32, [i32; 2], &mut [u8]) -> anyhow::Result<()>,
{
	for record in records {
		if read_string_from_record(record, "type").unwrap_or_default() == "Landscape" {
			//
			let grid_location = {
				let grid_location = record.get("grid").unwrap().as_array().unwrap();
				[
					grid_location[0].as_i64().unwrap() as i32,
					grid_location[1].as_i64().unwrap() as i32,
				]
			};

			let heights_base64 = record
				.get("vertex_heights")
				.unwrap()
				.get("data")
				.unwrap()
				.as_str()
				.unwrap();
			let mut heights_data = base64::prelude::BASE64_STANDARD
				.decode(heights_base64)
				.unwrap();
			const LANDSCAPE_HEIGHT_DATA_SIZE: usize = 4225;
			if heights_data.len() != LANDSCAPE_HEIGHT_DATA_SIZE {
				return Err(anyhow!(
					"Wrong landscape byte length, should be {} is {}",
					LANDSCAPE_HEIGHT_DATA_SIZE,
					heights_data.len()
				));
			}
			let mut height_offset = {
				if let Some(val) = record.get("vertex_heights").unwrap().get("offset") {
					//
					if let Some(val) = val.as_number() {
						val.as_f64().unwrap() as f32
					} else {
						0.0
					}
				} else {
					0.0
				}
			};

			closure(&mut height_offset, grid_location, &mut heights_data).unwrap();

			record
				.get_mut("vertex_heights")
				.unwrap()
				.as_object_mut()
				.unwrap()
				.insert("offset".to_string(), height_offset.into());
			*record
				.get_mut("vertex_heights")
				.unwrap()
				.get_mut("data")
				.unwrap() = base64::prelude::BASE64_STANDARD
				.encode(heights_data)
				.as_str()
				.into();
		}
	}

	Ok(())
}

fn validate_records_together(records: &mut Vec<Value>) -> anyhow::Result<()> {
	let mut openmw_world = OpenmwWorld::new();

	// Write data from landscapes to the openmw world
	do_for_all_landscapes(
		records,
		|height_offset: &mut f32, grid_location: [i32; 2], heights_data: &mut [u8]| {
			let mut offset = *height_offset;
			for iy in 0..65 {
				let mut row_offset = 0.0;
				for ix in 0..65 {
					let delta = i8::from_le_bytes([heights_data[iy * 65 + ix]]);
					if ix == 0 {
						offset += delta as f32;
					} else {
						row_offset += delta as f32;
					}

					openmw_world.set_elevation_canonical(
						[
							grid_location[0] * SQUARES_PER_CELL as i32 + ix as i32,
							grid_location[1] * SQUARES_PER_CELL as i32 + iy as i32,
						],
						offset + row_offset,
					)
				}
			}

			Ok(())
		},
	)
	.unwrap();

	// Read data from openmw world to landscapes
	do_for_all_landscapes(
		records,
		|height_offset: &mut f32, grid_location: [i32; 2], heights_data: &mut [u8]| {
			let mut last_pixel = openmw_world.get_elevation_canonical([
				grid_location[0] * SQUARES_PER_CELL as i32,
				grid_location[1] * SQUARES_PER_CELL as i32,
			]);
			let mut last_row = last_pixel;
			*height_offset = last_pixel as f32;
			for iy in 0..65 {
				for ix in 0..65 {
					let elevation = openmw_world.get_elevation_canonical([
						grid_location[0] * SQUARES_PER_CELL as i32 + ix as i32,
						grid_location[1] * SQUARES_PER_CELL as i32 + iy as i32,
					]);
					let delta: i32;
					if ix == 0 {
						delta = elevation - last_row;
						last_row = elevation;
						last_pixel = elevation;
					} else {
						delta = elevation - last_pixel;
						last_pixel = elevation;
					}
					if delta > i8::MAX as i32 || delta < i8::MIN as i32 {
						return Err(anyhow!(
							"HEIGHTMAP GRADIENT IS TOO LARGE, DELTA OUT OF I8 BOUNDS! ({} <= {} <= {})",
							i8::MIN, delta, i8::MAX
						));
					}
					heights_data[iy * 65 + ix] = i8::to_le_bytes(delta as i8)[0];
				}
			}

			Ok(())
		},
	)
	.unwrap();
	// panic!();

	// After all data is set, we can read it back into the buffers
	Ok(())
}

// Validates a single record upon reading
fn validate_single_record(record: &Value) -> anyhow::Result<()> {
	let record_type = read_string_from_record(record, "type").unwrap();
	match record_type.as_str() {
		"Header" => {
			let description = read_string_from_record(record, "description").unwrap();
			if description.len() > 256 {
				return Err(anyhow!("Description too long! Must be under 256 bytes!"));
			}

			let author = read_string_from_record(record, "author").unwrap();
			if author.len() > 32 {
				return Err(anyhow!("Author name too long! Must be under 32 bytes!"));
			}
		}
		_ => {}
	}

	let mesh_path = read_string_from_record(record, "mesh");
	if let anyhow::Result::Ok(mesh_path) = mesh_path {
		if mesh_path.len() == 0
			&& read_string_from_record(record, "type").unwrap_or_default() != "Npc"
		// npcs are allowed to have no mesh = they use racial meshes
		{
			let record_path = read_string_from_record(record, ORIGINAL_FILE_PATH_JSON_ATTR);
			return Err(anyhow!("Mesh not defined. Path: {:?}", record_path));
		}
		let mut path = std::env::current_dir().unwrap();
		path.push("assets/meshes");
		path.push(mesh_path.clone());
		path = get_target_path(&path);
		if !path.exists() {
			return anyhow::Result::Err(anyhow!(
				"Mesh path does not exist: {} ({})",
				mesh_path,
				path.to_string_lossy()
			));
		}
	}
	let icon_path = read_string_from_record(record, "icon");
	if let anyhow::Result::Ok(icon_path) = icon_path {
		if icon_path.len() == 0 {
			let record_path = read_string_from_record(record, ORIGINAL_FILE_PATH_JSON_ATTR);
			return Err(anyhow!("Icon not defined. Path: {:?}", record_path));
		}
		let mut path = std::env::current_dir().unwrap();
		path.push("assets/icons");
		path.push(icon_path.clone());
		path = get_target_path(&path);
		if !path.exists() {
			return anyhow::Result::Err(anyhow!(
				"Icon path does not exist: {} ({})",
				icon_path,
				path.to_string_lossy()
			));
		}
	}
	Ok(())
}

// Post-process a record by filling in values that we don't export to our custom json format
fn fill_in_single_record(
	record: &mut Value,
	record_count: usize,
	last_reference_index: &mut usize,
) -> anyhow::Result<()> {
	let record_type = read_string_from_record(record, "type").unwrap();

	match record_type.as_str() {
		"Header" => {
			if let Err(_) = read_string_from_record(record, "num_objects") {
				let obj = record.as_object_mut().unwrap(); //("num_objects");
				obj.insert("num_objects".into(), record_count.into());
			}
		}
		"Cell" => {
			let obj = record
				.get_mut("references")
				.unwrap()
				.as_array_mut()
				.unwrap();
			for o in obj {
				o.as_object_mut()
					.unwrap()
					.insert("mast_index".to_string(), 0.into());
				o.as_object_mut()
					.unwrap()
					.insert("refr_index".to_string(), (*last_reference_index).into());
				*last_reference_index += 1;
			}
		}
		"Landscape" => {
			let landscape_flags = record
				.get("landscape_flags")
				.expect("Landscape flags are compulsory but missing on a landscape record!")
				.as_str()
				.unwrap();
			if landscape_flags == "" {
				return Err(anyhow!("A landscape has an empty landscape flags string. This indicates an error. Perhaps the tool used to edit it has an internal bug.unwrap() A good default value is 'USES_VERTEX_HEIGHTS_AND_NORMALS | USES_TEXTURES'"));
			}
		}
		_ => {}
	}
	Ok(())
}
