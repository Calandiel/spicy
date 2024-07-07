use std::{
	env,
	fs::{self, OpenOptions},
	io::Write,
	path::Path,
};

// Creates a subdirectory
pub fn create_subdirectory(subpath: &str) -> anyhow::Result<()> {
	let mut path = env::current_dir()?;
	path.push(subpath);

	fs::create_dir_all(path.clone())?;

	Ok(())
}

// Creates a text file at a given subpath and writes it with content
pub fn create_text_file(subpath: &str, file: &str, content: &str) -> anyhow::Result<()> {
	create_bin_file(subpath, file, content.as_bytes())
}

// Creates an empty file at a given subpath
// pub fn create_empty_file(subpath: &str, file: &str) -> anyhow::Result<()> {
// create_text_file(subpath, file, "")
// }

// Creates a binary file at a given subpath and writes it with content
pub fn create_bin_file(subpath: &str, file: &str, content: &[u8]) -> anyhow::Result<()> {
	let mut path = env::current_dir().unwrap();

	create_subdirectory(subpath).unwrap();
	path.push(subpath);
	path.push(file);

	let mut file = OpenOptions::new()
		.write(true)
		.create(true)
		.truncate(true)
		.open(path.clone())
		.unwrap();

	file.write(content)?;

	Ok(())
}

pub fn copy_file_from_res_to_game_dir(
	path_src: &str,
	path_dst: &str,
	base_dst_path: &Option<String>,
) {
	let mut src = std::env::current_exe()
		.unwrap()
		.parent()
		.unwrap()
		.to_path_buf();
	let mut dst = std::env::current_dir().unwrap();
	if let Some(base_dst_path) = base_dst_path {
		dst.push(base_dst_path);
	}

	src.push(path_src);
	dst.push(path_dst);

	fs::copy(src, dst).unwrap();
}

pub fn copy_dir_from_res_to_game_dir(
	path_src: &str,
	path_dst: &str,
	only_if_doesnt_exist: bool,
	base_dst_path: Option<String>,
) {
	let src_base = std::env::current_exe()
		.unwrap()
		.parent()
		.unwrap()
		.to_path_buf();
	let mut dst_base = std::env::current_dir().unwrap();
	if let Some(base_dst_path) = base_dst_path {
		dst_base.push(base_dst_path);
	}

	let mut src = src_base.clone();
	src.push(path_src);
	let mut dst = dst_base.clone();
	dst.push(path_dst);

	if only_if_doesnt_exist && dst.exists() {
		return;
	}

	copy_dir_all(src, dst);
}

pub fn copy_dir_all(src: impl AsRef<Path>, dst: impl AsRef<Path>) {
	println!("Copying from: {:?} to {:?}", src.as_ref(), dst.as_ref());
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
