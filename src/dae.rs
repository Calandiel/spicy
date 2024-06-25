use std::{
	env,
	fs::{self, DirEntry},
};

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

fn read_meshes_from_obj(dir: &DirEntry) -> anyhow::Result<ObjScene> {
	let mut objects = vec![];
	let all_data = fs::read_to_string(dir.path())?;
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
				let uvx: f32 = words[1].parse()?;
				let uvy: f32 = words[2].parse()?;
				all_uvs.push([uvx, uvy]);
			} else {
				panic!("Wrong vt length!");
			}
			continue;
		}
		if words[0] == "vn" {
			if words.len() == 4 {
				let v_0: f32 = words[1].parse()?;
				let v_1: f32 = words[3].parse()?;
				let v_2: f32 = words[2].parse()?;
				all_normals.push([v_0, v_1, v_2]);
			} else {
				panic!("Wrong vn length!");
			}
			continue; // vertex normal
		}
		if words[0] == "v" {
			if words.len() == 4 {
				let v_0: f32 = words[1].parse()?;
				let v_1: f32 = words[3].parse()?;
				let v_2: f32 = words[2].parse()?;
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
					.parse::<usize>()? - 1];
				let t = all_uvs[words[i]
					.split('/')
					.skip(1)
					.next()
					.unwrap()
					.parse::<usize>()? - 1];
				let n = all_normals[words[i]
					.split('/')
					.skip(2)
					.next()
					.unwrap()
					.parse::<usize>()? - 1];
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

	Ok(ObjScene { roots: objects })
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

	Ok(dae_string)
}

pub fn convert_obj_to_dae() -> anyhow::Result<()> {
	let mut meshes_path = env::current_dir().unwrap();
	meshes_path.push("assets/meshes");

	println!("Mesh path: {:?}", meshes_path);
	visit_dirs(&meshes_path, &|dir| {
		println!("{:?}", dir);
		if dir.path().extension().unwrap_or_default() == "obj" {
			let objects = read_meshes_from_obj(dir).unwrap();
			let dae_string = convert_obj_list_to_dae_string(&objects).unwrap();
			let mut dae_path = dir.path().clone();
			dae_path.set_extension("dae");
			fs::write(dae_path, dae_string).unwrap();
		}
	})?;

	Ok(())
}

read_meshes_from_glb(dir: &DirEntry) -> anyhow::Result<ObjScene> {

	Ok(())
}

pub fn convert_glb_to_dae() -> anyhow::Result<()> {
	let mut meshes_path = env::current_dir().unwrap();
	meshes_path.push("assets/meshes");

	println!("Mesh path: {:?}", meshes_path);
	visit_dirs(&meshes_path, &|dir| {
		println!("{:?}", dir);
		if dir.path().extension().unwrap_or_default() == "glb" {
			let objects = read_meshes_from_glb(dir).unwrap();
			let dae_string = convert_obj_list_to_dae_string(&objects).unwrap();
			let mut dae_path = dir.path().clone();
			dae_path.set_extension("dae");
			fs::write(dae_path, dae_string).unwrap();
		}
	})?;

	Ok(())
}
// TODO: handle the case of glb and obj of the same name writing the same file... We should give an error if that happens...
