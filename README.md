# Spicy
This repository contains spicy - a simple validator built on top of tes3conv that splits records into individual files.

# Getting started
While tes3conv is included with the repo, a number of important dependencies aren't.
This section covers how to install them.

## Rust
To use spicy you need Rust installed and in your system path.
You can get it here: https://rustup.rs/

To verify that everything works, open your terminal, type in `cargo` and press `enter`.
You should be greeted with instructions on how to use cargo. If the system complains about a missing path or file, you forgot to add the folder containing it to your system path.
Note, if you are on Windows, you may need to restart the terminal or the system.

## OpenMW
To run projects, you will need the engine itself too.
You can download a version specific to your system from https://openmw.org/downloads/
Note that spicy was developed and tested with version 0.48 in mind.

It's highly suggested you create a new installation. After downloading the engine, DO NOT run the installation wizard that openmw ships with as we will not be pointing it to any external game files.

Instead, get `settings.cfg` from the repo and place it next to openmw executable. This will prepare it for use with spicy.

## git
To clone spicy and use it, install `git` and add it to your system path.
Further instructions can be found on https://git-scm.com/

## Building spicy
Clone the repo with `git` and run `cargo build --release`. After that, add `target/release` to your system path to ease usage of the tool.

## Creating a new project
This section will be written later as we are still working on automatic project creation.

## Exposing the project to openmw and openmw-cs
Run `openmw-launcher` and create a new profile.
Add your projects `common/build` directory to data paths.
This should be the *only* data path in the profile (besides the built-in path of the engine that can't be removed).

Lastly, edit `openmw.cfg`. Yes, even though it tells you not to (it's one of the reasons why you need a separate openmw installation).
`data-local` needs to point to `common/build` with a relative path, otherwise `openmw-cs` will not actually edit your mod file but instead create a cached copy on work on that (intended behavior, but why they'd want to break ctrl+s is beyond me).
For the developers of spicy the path is set to `data-local="../MorrowindGames/ApoMW/common/build"`

Enable the scripts and the addon of your game, then launch the game.
From then on, you can run the game with `openmw --skip-menu --new-game` for faster development, without going through the launcher.

# Intended workflow
Spicy doesn't come with editing tools.
You will need to rely on openmw-cs for that.

The general workflow is:
- pull changes from other contributors with `git pull`
- compile files with `spicy -a compile`
- edit files with `openmw-cs`
- save and debug wht game with `openmw --skip-menu --new-game`
- decompile files with `spicy -a decompile`
- commit and push with `git add --all && git commit -m "message" && git push`

## Asset workflow
Spicy handles conversion of standard glb files into a format compatible with openmw.
Put assets in your projects `assets/meshes` directory.
You can also put dae files in there but it's discouraged as glb is widely adapted and more commonly used.

The general workflow is:
- pull changes from other contributors with `git pull`
- compile files with `spicy -a compile`
- edit files with blender
- export them to glb and put them in `assets/meshes`
- save and debug wht game with `openmw --skip-menu --new-game`
- decompile files with `spicy -a decompile`
- commit and push with `git add --all && git commit -m "message" && git push`
