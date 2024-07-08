# Spicy
This repository contains spicy - a simple validator built on top of tes3conv that splits records into individual files.

# Building spicy
To build spicy you need to install a number of dependencies

## Rust
To use spicy you need Rust installed and in your system path.
You can get it here: https://rustup.rs/

To verify that everything works, open your terminal, type in `cargo` and press `enter`.
You should be greeted with instructions on how to use cargo. If the system complains about a missing path or file, you forgot to add the folder containing it to your system path.
Note, if you are on Windows, you may need to restart the terminal or the system.

## git
To clone spicy and use it, install `git` and add it to your system path.
Further instructions can be found here: https://git-scm.com/

## git-lfs
Some of the binary files spicy uses as dependencies are tracked using git-lfs.
Download and install it too: https://git-lfs.com/

## Building spicy
Clone the repo with `git` and run `cargo build --release`. After that, add `target/release` to your system path to ease usage of the tool.

```
git clone https://github.com/Calandiel/spicy.git
cd spicy
cargo build --release
```

## Creating a new project
Run `spicy new <project name>`

# Intended workflow
Spicy doesn't come with editing tools.
You will need to rely on openmw-cs for that.

The general workflow is:
- pull changes from other contributors with `git pull`
- compile files with `spicy compile`
- edit files with `openmw-cs`
- decompile files with `spicy decompile`
- test the game with `spicy run`
- commit and push with `git add --all && git commit -m "message" && git push`

## Asset workflow
Spicy handles conversion of standard glb files into a format compatible with openmw.
Put assets in your projects `assets/meshes` directory.
You can also put dae files in there but it's discouraged as glb is widely adapted and more commonly used.

The general workflow is:
- pull changes from other contributors with `git pull`
- compile files with `spicy compile`
- edit files with blender
- export them to glb and put them in `assets/meshes`
- decompile files with `spicy decompile`
- test the game with `spicy run`
- commit and push with `git add --all && git commit -m "message" && git push`
