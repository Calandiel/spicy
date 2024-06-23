use anyhow::{anyhow, Ok};
use clap::Parser;
use serde_json::{from_str, Value};
use std::{
	env,
	fs::{self, OpenOptions},
	io::Write,
	path::PathBuf,
	process::{Command, Stdio},
	sync::Arc,
};

const TODD_UNIT: f32 = 70.0; // the scaling from meters to todd units

fn get_record_types() -> Vec<String> {
	vec![
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
		//"DialogueInfo".to_string(), // Handled differently... To simplify usage, we embed them on dialogue records.
	]
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

	ensure_common_exists()?;

	match args.action {
		Action::New => {
			new()?;
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

fn process_directory(mut input_path: PathBuf, outputs: &mut Vec<PathBuf>) -> anyhow::Result<()> {
	if cfg!(target_os = "windows") {
		let new_input_path = input_path.clone().to_string_lossy().replace("/", "\\");
		input_path = new_input_path.into();
	}
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

	let _ = ensure_tes3conv_exists();

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
		let mut windows_path = env::current_dir().unwrap();
		windows_path.push("common/tes3conv/windows/tes3conv.exe");
		let mut file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(windows_path.clone())?;
		file.write_all(tes3conv_windows)?;

		let mut linux_path = env::current_dir().unwrap();
		linux_path.push("common/tes3conv/ubuntu/tes3conv");
		let mut file = OpenOptions::new()
			.write(true)
			.create(true)
			.truncate(true)
			.open(linux_path.clone())?;
		file.write_all(tes3conv_linux)?;

		set_exe_permissions(linux_path.clone())?;
	}

	Ok(())
}

#[cfg(target_os = "linux")]
fn set_exe_permissions(linux_path: PathBuf) -> anyhow::Result<()> {
	use std::os::unix::fs::PermissionsExt;
	let metadata = std::fs::metadata(linux_path.clone())?;
	let mut permissions = metadata.permissions();
	permissions.set_mode(0o775);
	fs::set_permissions(linux_path, permissions)?;

	Ok(())
}

#[cfg(target_os = "windows")]
fn set_exe_permissions(linux_path: PathBuf) -> anyhow::Result<()> {
	println!(
		"not on linux so we're not setting permissions on {:?}",
		linux_path
	);
	Ok(())
}

fn new() -> anyhow::Result<()> {
	clear()?;
	create_record_dirs()?;

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

	let mut path = env::current_dir().unwrap();
	path.push("common/data/Header/header.json");
	let mut file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(path)?;
	file.write_all(json.as_bytes())?;

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

	create_gitkeep("common/cache", ".gitkeep")?;
	create_gitkeep("common/build", ".gitkeep")?;
	create_gitkeep("common/data", ".gitkeep")?;
	ensure_tes3conv_exists()?;

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

	let mut cell_reference_counter = 0;
	let mut dialogue_info_id_counter = 0;
	let mut json = r"[".to_string();
	for (index, file) in files.iter().enumerate() {
		println!("Parsing: {:?}", file);
		let json_data = fs::read_to_string(file).unwrap();
		let mut parsed_json: Value = from_str(&json_data).expect("Invalid JSON");

		// Validate individual records...
		fill_in_single_record(&mut parsed_json, files.len(), &mut cell_reference_counter)?;
		validate_single_record(&parsed_json)?;

		// Write the final value

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
				fill_in_single_record(info, 0, &mut cell_reference_counter)?;
				validate_single_record(info)?;

				json.push_str(",\n");
				json.push_str(&info.to_string());
			}
		}
		if index != files.len() - 1 {
			json.push_str(",\n");
		}
	}
	json.push(']');

	println!("Saving final json...");
	let mut file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(temporary_json_path.clone())?;
	file.write_all(json.as_bytes())?;

	println!("Running: {:?}\n", tes3conv_path);
	let output = Command::new(tes3conv_path)
		.arg(temporary_json_path.to_string_lossy().to_string())
		.arg(output_path.to_string_lossy().to_string())
		.stdout(Stdio::inherit())
		.stderr(Stdio::inherit())
		.output()
		.expect("Failed to tes3conv");

	println!("{:?}", output);

	println!("Converting .obj files to .dae");
	convert_obj_to_dae()?;

	Ok(())
}

#[derive(Default, Debug)]
struct ObjMesh {
	name: String,
	vertices: Vec<[f32; 3]>,
	normals: Vec<[f32; 3]>,
	uvs: Vec<[f32; 2]>,
	triangles: Vec<i32>,
}

fn convert_obj_to_dae() -> anyhow::Result<()> {
	let mut meshes_path = env::current_dir().unwrap();
	meshes_path.push("assets/meshes");

	println!("Mesh path: {:?}", meshes_path);
	visit_dirs(&meshes_path, &|dir| {
		println!("{:?}", dir);
		if dir.path().extension().unwrap_or_default() == "obj" {
			let mut objects = vec![];
			let all_data = fs::read_to_string(dir.path()).unwrap();
			let mut all_vertices: Vec<[f32; 3]> = vec![];
			let mut all_normals: Vec<[f32; 3]> = vec![];
			let mut all_uvs: Vec<[f32; 2]> = vec![];
			for line in all_data.lines() {
				let words: Vec<&str> = line.split_ascii_whitespace().to_owned().collect();
				if words[0] == "#" {
					continue; // comment, skip
				}
				if words[0] == "mtllib" {
					continue; // material, ignore
				}
				if words[0] == "o" {
					objects.push(ObjMesh::default());
					objects.last_mut().unwrap().name = words[1].to_string();
					continue; // object uwu
				}
				if words[0] == "vt" {
					if words.len() == 3 {
						let uvx: f32 = words[1].parse().unwrap();
						let uvy: f32 = words[2].parse().unwrap();
						all_uvs.push([uvx, uvy]);
					} else {
						panic!("Wrong vt length!");
					}
					continue;
				}
				if words[0] == "vn" {
					if words.len() == 4 {
						let v_0: f32 = words[1].parse().unwrap();
						let v_1: f32 = words[3].parse().unwrap();
						let v_2: f32 = words[2].parse().unwrap();
						all_normals.push([v_0, v_1, v_2]);
					} else {
						panic!("Wrong vn length!");
					}
					continue; // vertex normal
				}
				if words[0] == "v" {
					if words.len() == 4 {
						let v_0: f32 = words[1].parse().unwrap();
						let v_1: f32 = words[3].parse().unwrap();
						let v_2: f32 = words[2].parse().unwrap();
						all_vertices.push([v_0, v_1, v_2]);
					} else {
						panic!("Wrong v length!");
					}
					continue; // vertex position
				}
				if words[0] == "f" {
					for i in 1..words.len() {
						let v = all_vertices[words[i]
							.split('/')
							.skip(0)
							.next()
							.unwrap()
							.parse::<usize>()
							.unwrap() - 1];
						let t = all_uvs[words[i]
							.split('/')
							.skip(1)
							.next()
							.unwrap()
							.parse::<usize>()
							.unwrap() - 1];
						let n = all_normals[words[i]
							.split('/')
							.skip(2)
							.next()
							.unwrap()
							.parse::<usize>()
							.unwrap() - 1];
						objects.last_mut().unwrap().vertices.push(v);
						objects.last_mut().unwrap().normals.push(n);
						objects.last_mut().unwrap().uvs.push(t);
					}
					let vc = objects.last().unwrap().vertices.len() as i32;
					if words.len() == 4 {
						objects.last_mut().unwrap().triangles.push(vc - 3);
						objects.last_mut().unwrap().triangles.push(vc - 1);
						objects.last_mut().unwrap().triangles.push(vc - 2);
					} else if words.len() == 5 {
						objects.last_mut().unwrap().triangles.push(vc - 4);
						objects.last_mut().unwrap().triangles.push(vc - 2);
						objects.last_mut().unwrap().triangles.push(vc - 3);

						objects.last_mut().unwrap().triangles.push(vc - 4);
						objects.last_mut().unwrap().triangles.push(vc - 1);
						objects.last_mut().unwrap().triangles.push(vc - 2);
					} else {
						panic!("Wrong face length!");
					}
					continue; // face
				}
				if words[0] == "s" {
					continue; // ? ignore
				}
			}

			// After reading the obj file, convert to .dae
			let mut dae_string: String = "".to_string();
			dae_string.push_str(
				r###"<?xml version="1.0" encoding="utf-8"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
<asset>
	<contributor>
		<author>Spicy</author>
		<authoring_tool>Spicy converter for OpenMW</authoring_tool>
	</contributor>
	<created>2022-01-31T22:16:14Z</created>
	<modified>2022-01-31T22:16:14Z</modified>
	<unit meter="1.0" name="meter"/>
	<up_axis>Z_UP</up_axis>
</asset>"###,
			);
			dae_string.push_str(
				r###"
<library_images>
</library_images>
<library_effects>
</library_effects>
<library_materials>
</library_materials>
<library_geometries>"###,
			);
			for (idx, obj) in objects.iter().enumerate() {
				dae_string.push_str(
					format!(
						r###"
	<geometry id="id-{}-{}" name="{}">
		<mesh>
			<source id="id-{}-{}-positions">
				<float_array id="id-{}-{}-positions-array" count="{}">
					"###,
						obj.name,
						idx,
						obj.name,
						obj.name,
						idx,
						obj.name,
						idx,
						obj.vertices.len() * 3
					)
					.as_str(),
				);
				for v in &obj.vertices {
					for i in 0..3 {
						dae_string.push_str(" ");
						let vv = v[i] * TODD_UNIT;
						dae_string.push_str(vv.to_string().as_str());
					}
				}
				dae_string.push_str(
					format!(
						r###"
				</float_array>
				<technique_common>
				<accessor source="#id-{}-{}-positions-array" count="{}" stride="3">
					<param name="X" type="float"/>
					<param name="Y" type="float"/>
					<param name="Z" type="float"/>
				</accessor>
				</technique_common>
			</source>
			<source id="id-{}-{}-normals">
				<float_array id="id-{}-{}-normals-array" count="{}">
					"###,
						obj.name,
						idx,
						obj.vertices.len(),
						obj.name,
						idx,
						obj.name,
						idx,
						obj.normals.len() * 3,
					)
					.as_str(),
				);
				for n in &obj.normals {
					for i in 0..3 {
						dae_string.push_str(" ");
						dae_string.push_str(n[i].to_string().as_str());
					}
				}
				dae_string.push_str(
					format!(
						r###"
				</float_array>
				<technique_common>
				<accessor source="#id-{}-{}-normals-array" count="{}" stride="3">
					<param name="X" type="float"/>
					<param name="Y" type="float"/>
					<param name="Z" type="float"/>
				</accessor>
				</technique_common>
			</source>
			<source id="id-{}-{}-texcoord-0">
				<float_array id="id-{}-{}-texcoord-0-array" count="{}">
					"###,
						obj.name,
						idx,
						obj.normals.len(),
						obj.name,
						idx,
						obj.name,
						idx,
						obj.uvs.len() * 2
					)
					.as_str(),
				);
				for t in &obj.uvs {
					for i in 0..2 {
						dae_string.push_str(" ");
						dae_string.push_str(t[i].to_string().as_str());
					}
				}
				dae_string.push_str(
					format!(
						r###"
				</float_array>
				<technique_common>
				<accessor source="#id-{}-{}-texcoord-0-array" count="{}" stride="2">
					<param name="S" type="float"/>
					<param name="T" type="float"/>
				</accessor>
				</technique_common>
			</source>
			<vertices id="id-{}-{}-vertices">
				<input semantic="POSITION" source="#id-{}-{}-positions"/>
			</vertices>
			<triangles count="{}" >
				<input semantic="VERTEX" source="#id-{}-{}-vertices" offset="0"/>
				<input semantic="NORMAL" source="#id-{}-{}-normals" offset="0"/>
				<input semantic="TEXCOORD" source="#id-{}-{}-texcoord-0" offset="0" set="0"/>
				<p>"###,
						obj.name,
						idx,
						obj.uvs.len(),
						obj.name,
						idx,
						obj.name,
						idx,
						obj.triangles.len() / 3,
						obj.name,
						idx,
						obj.name,
						idx,
						obj.name,
						idx,
					)
					.as_str(),
				); // in triangles count skipped this: `material="id-trimat-9"`
				for i in &obj.triangles {
					dae_string.push_str(" ");
					dae_string.push_str(i.to_string().as_str());
				}
				dae_string.push_str(
					format!(
						r###"
				</p>
			</triangles>
		</mesh>
	</geometry>"###
					)
					.as_str(),
				);
			}
			dae_string.push_str(
				r###"
</library_geometries>
<library_visual_scenes>
	<visual_scene id="id-scene-1" name="scene">"###,
			);
			for (idx, obj) in objects.iter().enumerate() {
				if obj.name == "collisionmesh" {
					dae_string.push_str(
						format!(
							r###"
		<node id="collision" name="collision" type="NODE">
			<matrix sid="transform"> 1.0 0.0 0.0 0.0 0.0 1.0 0.0 0.0 0.0 0.0 1.0 0.0 0.0 0.0 0.0 1.0  </matrix>
				<extra>
					<technique profile="GODOT">
						<empty_draw_type>PLAIN_AXES</empty_draw_type>
					</technique>
				</extra>
			<node id="{}" name="{}" type="NODE">
				<matrix sid="transform"> 1.0 0.0 0.0 0.0 0.0 1.0 0.0 0.0 0.0 0.0 1.0 0.0 0.0 0.0 0.0 1.0  </matrix>
				<instance_geometry url="#id-{}-{}">
				</instance_geometry>
			</node>
		</node>"###,
							obj.name, obj.name, obj.name, idx
						)
						.as_str(),
					);
				} else {
					dae_string.push_str(
						format!(
							r###"
		<node id="{}" name="{}" type="NODE">
			<matrix sid="transform"> 1.0 0.0 0.0 0.0 0.0 1.0 0.0 0.0 0.0 0.0 1.0 0.0 0.0 0.0 0.0 1.0  </matrix>
			<instance_geometry url="#id-{}-{}">
			</instance_geometry>
		</node>"###,
							// removed this from within instance_geometry:
							/*
							<bind_material>
								<technique_common>
									<instance_material symbol="id-trimat-9" target="#id-material-7"/>
								</technique_common>
							</bind_material>
							*/
							obj.name,
							obj.name,
							obj.name,
							idx
						)
						.as_str(),
					);
				}
			}
			dae_string.push_str(
				r###"
	</visual_scene>
</library_visual_scenes>
<scene>
	<instance_visual_scene url="#id-scene-1" />
</scene>
</COLLADA>"###,
			);
			let mut dae_path = dir.path().clone();
			dae_path.set_extension("dae");
			fs::write(dae_path, dae_string).unwrap();
		}
	})?;

	Ok(())
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(dir: &std::path::Path, cb: &dyn Fn(&fs::DirEntry)) -> anyhow::Result<()> {
	if dir.is_dir() {
		for entry in fs::read_dir(dir)? {
			let entry = entry?;
			let path = entry.path();
			if path.is_dir() {
				visit_dirs(&path, cb)?;
			} else {
				cb(&entry);
			}
		}
	}

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
		file.write_all(&data)?;

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

	create_record_dirs()?;

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
		// let decoded = BASE64_STANDARD.decode(data_in_json.as_str().unwrap())?;
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
		let stringified_record = serde_json::to_string_pretty(record)?;
		let data_to_write = stringified_record.as_bytes();
		file?.write_all(data_to_write)?;
	}

	// At the end, delete the file as we won't need it anymore.
	if output_path.clone().exists() {
		fs::remove_file(output_path.clone()).unwrap();
	}

	Ok(())
}

fn create_record_dirs() -> anyhow::Result<()> {
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

		validate_single_record(record)?;
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

// Validates a single record upon reading
fn validate_single_record(record: &Value) -> anyhow::Result<()> {
	let record_type = read_string_from_record(record, "type")?;
	match record_type.as_str() {
		"Header" => {
			let description = read_string_from_record(record, "description")?;
			if description.len() > 256 {
				return Err(anyhow!("Description too long! Must be under 256 bytes!"));
			}

			let author = read_string_from_record(record, "author")?;
			if author.len() > 32 {
				return Err(anyhow!("Author name too long! Must be under 32 bytes!"));
			}
		}
		_ => {}
	}
	Ok(())
}

// Post-process a record by filling in values that we don't export to our specific json format
fn fill_in_single_record(
	record: &mut Value,
	record_count: usize,
	last_reference_index: &mut usize,
) -> anyhow::Result<()> {
	let record_type = read_string_from_record(record, "type")?;

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
		_ => {}
	}
	Ok(())
}
