use std::{
	env, fs,
	path::{Path, PathBuf},
};

fn main() {
	// Re-runs script if any files in res are changed
	println!("cargo:rerun-if-changed=res/*");
	copy_single_folder("res");
}

fn copy_single_folder(relative_path: &str) {
	let input_path = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join(relative_path);
	let output_path = get_output_path();
	let output_path = Path::new(&output_path).join(relative_path);
	if output_path.exists() {
		fs::remove_dir_all(&output_path).unwrap();
	}
	copy_dir_all(input_path, output_path);
}

fn get_output_path() -> PathBuf {
	//<root or manifest path>/target/<profile>/
	let manifest_dir_string = env::var("CARGO_MANIFEST_DIR").unwrap();
	let build_type = env::var("PROFILE").unwrap();
	let path = Path::new(&manifest_dir_string)
		.join("target")
		.join(build_type);
	return PathBuf::from(path);
}

fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
	fs::create_dir_all(&dst).unwrap();
	for entry in fs::read_dir(src).unwrap() {
		let entry = entry.unwrap();
		let ty = entry.file_type().unwrap();
		if ty.is_dir() {
			copy_dir_all(entry.path(), dst.as_ref().join(entry.file_name()));
		} else {
			fs::copy(entry.path(), dst.as_ref().join(entry.file_name())).unwrap();
		}
	}
}
