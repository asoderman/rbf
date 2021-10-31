rbf: a minimal brainfuck -> x86 compiler written in rust.

rbf attempts to generate (readable) amd64 assembly which is then fed to nasm with the resulting object files linked with ld.

Currently rbf only creates binaries that support macos. The assembly in theory should be portable to linux as the only platform specific code is printf(see note) which is invoked from libc. This is assuming the calling convention is the same. However I currently do not have access to a linux environment to verify this.

Note: as of right now getchar (,) is not implemented. This will also likely be platform specific.
