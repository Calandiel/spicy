use std::{
	env,
	fs::{self, OpenOptions},
	io::Write,
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
pub fn create_empty_file(subpath: &str, file: &str) -> anyhow::Result<()> {
	create_text_file(subpath, file, "")
}

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
