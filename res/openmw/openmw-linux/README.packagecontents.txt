=== Package Content README ===

This package contains Linux binaries for openmw and its suite of utilities. It is intended to run both on most moderatly old and recent Linux distributions. Basically anything as old or newer than Debian 12 (Bookworm).

=== Running the provided binaries ===
The binaries depend on libraries that might not be installed on your system or in an incompatible version. Therefore these libraries are included in the "lib" subdirectory. To automatically use the provided libraries there are scripts which set the LD_LIBRARY_PATH variable to the "lib" directory.

To run openmw or an utility simply run the provided shell script (the file without a suffix -> <filename> not <filename>.x86_64). In case of openmw this would be "openmw" not openmw.x86_64.
