#[macro_use]
extern crate maplit;
#[macro_use]
extern crate lazy_static;
use std::env;

mod compiler;
mod interpretter;

use interpretter::Interpretter;

use std::io::Result;
use std::process::Command;

use argparse::{ArgumentParser, StoreTrue, Store};

#[derive(Default)]
struct Config {
    keep_asm: bool,
    keep_o: bool
}

fn main() -> Result<()> {
    let mut filename = String::new();
    let mut config = Config::default();
    {
        let mut ap = ArgumentParser::new();
        ap.set_description("Compile a brainfuck program to x86_64 binaries.");
        ap.refer(&mut filename)
            .add_argument("filename", Store, "The file to be compiled.");
        ap.refer(&mut config.keep_asm)
            .add_option(&["--asm"], StoreTrue, "Keep nasm assembly files.");
        ap.refer(&mut config.keep_o)
            .add_option(&["--object"], StoreTrue, "Keep object files generated by nasm.");

        ap.parse_args_or_exit();
    }
    let input_string = std::fs::read_to_string(&filename)
        .expect(&format!("Could not open file: {}", &filename));
    let bf = compiler::Compilation::new(input_string);

    std::fs::write("out.asm", &bf.generate_assembly())?;
    println!("[Compiler]: Assembly generated to out.asm");

    nasm_assemble(&filename);
    macos_ld_link(&filename);
    asm_o_cleanup(&filename, &config);

    Ok(())
}

fn nasm_assemble(filename: &str) {
    let assembly_step = Command::new("nasm")
        .arg("-f")
        .arg("macho64")
        .arg("out.asm")
        .output()
        .expect("Could not run assembler (nasm)");

    println!("[Command]: nasm: {}", assembly_step.status);
    if !assembly_step.status.success() {
        println!("[Command]: nasm exited with message: {:#?}", assembly_step);
    }
}

fn macos_ld_link(filename: &str) {
    let link_step = Command::new("ld")
        .arg("out.o")
        .arg("-o")
        .arg(format!("{}", filename.split(".").nth(0).unwrap()))
        .arg("-macosx_version_min")
        .arg("10.13")
        .arg("-lSystem")
        .arg("-L/Library/Developer/CommandLineTools/SDKs/MacOSX.sdk/usr/lib")
        .output()
        .expect("Could not run linker (ld)");

    println!("[Command]: ld: {}", link_step.status);
    if !link_step.status.success() {
        println!("[Command]: ld exited with message: {:#?}", link_step);
    } else {
    println!("[Compiler]: Artifact '{}' created", filename.split(".").nth(0).unwrap());
    }

}

fn asm_o_cleanup(filename: &str, config: &Config) {
    if !config.keep_o {
        let o_cleanup_step = Command::new("rm")
            .arg("out.o")
            .output()
            .expect("Could not remove .o file");
    }

    if !config.keep_asm {
        let asm_cleanup = Command::new("rm")
            .arg("out.asm")
            .output()
            .expect("Could not remove .asm file");
    }
}