use std::{
	collections::HashSet,
	env,
	fs::{self, DirEntry},
	path::PathBuf,
};

use anyhow::anyhow;
use gltf::Node;

use crate::constants::TODD_UNIT;

#[derive(Default, Debug)]
pub struct ObjScene {
	pub roots: Vec<ObjMesh>,
}

#[derive(Default, Debug)]
pub struct ObjMesh {
	pub name: String,
	pub children: Vec<Box<ObjMesh>>,
	pub vertices: Vec<[f32; 3]>,
	pub normals: Vec<[f32; 3]>,
	pub uvs: Vec<[f32; 2]>,
	pub triangles: Vec<i32>,
	pub transform: [[f32; 4]; 4],
}

// one possible implementation of walking a directory only visiting files
fn visit_dirs(
	dir: &std::path::Path,
	set: &mut HashSet<PathBuf>,
	cb: &dyn Fn(&fs::DirEntry, &mut HashSet<PathBuf>) -> anyhow::Result<()>,
) -> anyhow::Result<()> {
	if dir.is_dir() {
		for entry in fs::read_dir(dir).unwrap() {
			let entry = entry.unwrap();
			let path = entry.path();
			if path.is_dir() {
				visit_dirs(&path, set, cb).unwrap();
			} else {
				cb(&entry, set).unwrap();
			}
		}
	}

	Ok(())
}

fn convert_obj_list_to_dae_string(objects: &ObjScene) -> anyhow::Result<String> {
	let mut dae_string: String = "".to_string();
	dae_string.push_str(
		r###"<?xml version="1.0" encoding="utf-8"?>
<COLLADA xmlns="http://www.collada.org/2005/11/COLLADASchema" version="1.4.1">
<asset>
	<contributor>
		<author>Spicy</author>
		<authoring_tool>Spicy converter for OpenMW</authoring_tool>
	</contributor>
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
	for (idx, obj) in objects.roots.iter().enumerate() {
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
				let vv = v[i] * crate::constants::TODD_UNIT;
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
		if obj.uvs.len() == 0 {
			for _ in &obj.vertices {
				dae_string.push_str(" 0 0");
			}
		} else if obj.uvs.len() != obj.vertices.len() {
			return Err(anyhow!("Mismatching uv and vertex buffers found!"));
		} else {
			for t in &obj.uvs {
				for i in 0..2 {
					dae_string.push_str(" ");
					dae_string.push_str(t[i].to_string().as_str());
				}
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
	for (idx, obj) in objects.roots.iter().enumerate() {
		let mut transform_string = "".to_string();
		for k in 0..4 {
			for l in 0..4 {
				let mul = if l == 3 && k != 3 { TODD_UNIT } else { 1.0 };
				transform_string += &(mul * obj.transform[l][k]).to_string();
				transform_string += " ";
			}
		}
		println!("TRANSFORM: {:?}", transform_string);
		if obj.name == "collisionmesh" {
			dae_string.push_str(
				format!(
					r###"
		<node id="collision" name="collision" type="NODE">
			<matrix sid="transform"> {} </matrix>
				<extra>
					<technique profile="GODOT">
						<empty_draw_type>PLAIN_AXES</empty_draw_type>
					</technique>
				</extra>
			<node id="{}" name="{}" type="NODE">
				<matrix sid="transform"> {} </matrix>
				<instance_geometry url="#id-{}-{}">
				</instance_geometry>
			</node>
		</node>"###,
					transform_string, obj.name, obj.name, transform_string, obj.name, idx
				)
				.as_str(),
			);
		} else {
			dae_string.push_str(
				format!(
					r###"
		<node id="{}" name="{}" type="NODE">
			<matrix sid="transform"> {}  </matrix>
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
					transform_string,
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

	Ok(dae_string)
}

fn rotate_to_z_up(matrix: &mut [[f32; 4]; 4]) {
	let rotation_matrix = [
		[1.0, 0.0, 0.0, 0.0],
		[0.0, 0.0, 1.0, 0.0],
		[0.0, -1.0, 0.0, 0.0],
		[0.0, 0.0, 0.0, 1.0],
	];

	*matrix = multiply_matrices(matrix, &rotation_matrix);
}

fn multiply_matrices(a: &[[f32; 4]; 4], b: &[[f32; 4]; 4]) -> [[f32; 4]; 4] {
	let mut result = [[0.0; 4]; 4];

	for i in 0..4 {
		for j in 0..4 {
			for k in 0..4 {
				result[i][j] += a[i][k] * b[k][j];
			}
		}
	}

	result
}

fn walk_down_gltf(
	node: &Node,
	buffers: &Vec<gltf::buffer::Data>,
	images: &Vec<gltf::image::Data>,
	parent: Option<&mut ObjMesh>,
	scene: &mut ObjScene,
) {
	let mut this_node = ObjMesh::default();

	for child in node.children() {
		walk_down_gltf(&child, buffers, images, Some(&mut this_node), scene);
	}

	// TODO: transforms, material, name collisions, animations, bones, morph targets, uvs, handle non triangular primitives...
	// let transform = node.transform();
	this_node.name = node.name().unwrap().to_string();

	let mut first = true;
	if let Some(mesh) = node.mesh() {
		for prim in mesh.primitives() {
			let reader = prim.reader(|buffer| Some(&buffers[buffer.index()]));
			let positions = reader
				.read_positions()
				.expect("A MESH IS MISSING POSITIONS!");
			let normals = reader.read_normals().expect("A MESH IS MISSING NORMALS!");
			let indices = reader.read_indices().expect("A MESH IS MISSING INDICES!");
			let transform = node.transform();
			let (_, _, scale) = transform.decomposed();
			let mut matrix = node.transform().matrix();

			// uwu
			let mut new_node_option = if !first {
				Some(ObjMesh::default())
			} else {
				None
			};

			let node_to_write = if first {
				&mut this_node
			} else {
				new_node_option.as_mut().unwrap()
			};

			// fill the mesh with data
			for position in positions {
				node_to_write
					.vertices
					.push([position[0], position[1], position[2]])
				// .push([position[0], position[2], position[1]])
			}
			for normal in normals {
				node_to_write.normals.push([
					scale[0].signum() * normal[0],
					scale[1].signum() * normal[1],
					scale[2].signum() * normal[2],
				]);
				// .push([normal[0], normal[2], normal[1]]);
			}
			let indices: Vec<i32> = indices.into_u32().map(|i| i as i32).collect();
			for i in indices.chunks_exact(3) {
				node_to_write.triangles.push(i[0]);
				node_to_write.triangles.push(i[1]);
				node_to_write.triangles.push(i[2]);
			}

			rotate_to_z_up(&mut matrix);
			node_to_write.transform = matrix;

			// more borrow juggling!
			if first {
				first = false;
			} else {
				this_node.children.push(Box::new(new_node_option.unwrap()));
			}
		}
	}

	// At the end, assign the new node to the scene roots or the parent
	if let Some(parent) = parent {
		parent.children.push(Box::new(this_node));
	} else {
		scene.roots.push(this_node);
	}
}

fn read_meshes_from_glb(dir: &DirEntry) -> anyhow::Result<ObjScene> {
	// gltf::r
	let (gltf, buffers, images) = gltf::import(dir.path())?;
	let mut obj_scene = ObjScene { roots: vec![] };
	for scene in gltf.scenes() {
		for node in scene.nodes() {
			walk_down_gltf(&node, &buffers, &images, None, &mut obj_scene);
		}
	}

	Ok(obj_scene)
}

// Replaces a part of a path string corresponding to the input directory with one corresponding to the target directory for assets
pub fn get_target_path(original: &PathBuf) -> PathBuf {
	let mut meshes_path = env::current_dir().unwrap();
	meshes_path.push("assets");

	let mut meshes_target = env::current_dir().unwrap();
	meshes_target.push("build");

	let original_str = original.to_str().unwrap();
	return original_str
		.replace(
			meshes_path.to_str().unwrap(),
			meshes_target.to_str().unwrap(),
		)
		.into();
}

fn copy_directory(subdir: &str) -> anyhow::Result<()> {
	let mut dir_path = std::env::current_dir()?;
	dir_path.push(subdir);
	let mut all_icon_paths = std::collections::hash_set::HashSet::new();
	visit_dirs(&dir_path, &mut all_icon_paths, &|dir, _| {
		let file_content = fs::read(dir.path())?;
		fs::create_dir_all(get_target_path(&dir.path()).parent().unwrap()).unwrap();
		fs::write(get_target_path(&dir.path()), file_content).unwrap();
		Ok(())
	})?;

	Ok(())
}

pub fn compile_assets() -> anyhow::Result<()> {
	copy_directory("assets/icons")?;
	copy_directory("assets/textures")?;
	copy_directory("assets/music")?;
	copy_directory("assets/sound")?;
	copy_directory("assets/scripts")?;

	let file_content = {
		let mut out_script_path = std::env::current_dir()?;
		out_script_path.push("assets/out.omwscripts");
		if let Ok(e) = fs::read(out_script_path) {
			Some(e)
		} else {
			None
		}
	};
	if let Some(file_content) = file_content {
		let mut out_script_path = std::env::current_dir()?;
		out_script_path.push("build/out.omwscripts");
		fs::create_dir_all(get_target_path(&out_script_path).parent().unwrap())?;
		fs::write(get_target_path(&out_script_path), file_content).unwrap();
	}

	let mut all_mesh_paths = std::env::current_dir()?;
	all_mesh_paths.push("assets/meshes");
	let mut all_paths = std::collections::hash_set::HashSet::new();
	println!("Mesh path: {:?}", all_mesh_paths);
	visit_dirs(&all_mesh_paths, &mut all_paths, &|dir, all_paths| {
		println!("{:?}", dir);
		let dir_path = dir.path();
		let extension = dir_path.extension().unwrap_or_default();
		if extension == "glb" {
			// Dae files need to be preprocessed before being moved to another location
			let objects = read_meshes_from_glb(dir).unwrap();
			let dae_string = convert_obj_list_to_dae_string(&objects).unwrap();
			let mut dae_path = dir_path.clone();
			dae_path.set_extension("dae");
			if !all_paths.insert(get_target_path(&dae_path)) {
				return Err(anyhow!("Duplicate file: {}", dae_path.to_string_lossy()))?;
			}
			fs::create_dir_all(get_target_path(&dae_path).parent().unwrap())?;
			fs::write(get_target_path(&dae_path), dae_string).unwrap();
		} else if extension == "dae" || extension == "txt" || extension == "osgt" {
			// For dae/txt/osgt files, just copy them over
			let file_content = fs::read(dir_path.clone())?;
			let dae_path = dir_path.clone();
			if !all_paths.insert(get_target_path(&dae_path)) {
				return Err(anyhow!("Duplicate file: {}", dae_path.to_string_lossy()))?;
			}
			fs::create_dir_all(get_target_path(&dae_path).parent().unwrap())?;
			fs::write(get_target_path(&dae_path), file_content).unwrap();
		}
		Ok(())
	})?;

	Ok(())
}
