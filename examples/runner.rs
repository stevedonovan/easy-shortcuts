// a simple Rust 'script' runner.
extern crate easy_shortcuts as es;
use es::traits::*;
use std::process;
use std::env;
use std::fs;
use std::path::PathBuf;

fn rustup_lib() -> String {
    let rustup_toolchain = es::shell("rustup toolchain list");
    // get the active toolchain
    let tchain = rustup_toolchain.lines()
        .find(|s| s.ends_with("(default)"))
        .or_die("no Rust toolchain").to_string();
    // and strip off the '(default)'
    let toolchain =  tchain.split_at_delim(' ').unwrap().0;
    let home = env::home_dir().unwrap();
    format!("{}/.rustup/toolchains/{}/lib",home.display(),toolchain)
}

fn found(hay: &str, needle: &str) -> bool {
    hay.find(needle).is_some()
}

fn main() {
    let temp = "temp";
    if ! fs::metadata(temp).is_dir() {
        fs::create_dir(temp).or_die("cannot create temp directory here");
    }

    let file = PathBuf::from(es::argn_err(1,"please supply a source file"));
    let ext = file.extension().or_die("no file extension");
    if ext != "rs" {
        es::quit("file extension must be .rs");
    }

    let code = es::read_to_string(&file);

    let mut prefix = String::new();
    if found(&code,"File::") {
        prefix += "use std::fs::File;\nuse std::io::prelude::*;\n";
    }
    if found(&code,"env::").is_some() {
        prefix += "use std::env;\n";
    }
    if found(&code,"PathBuf::").is_some() {
        prefix += "use std::path:{PathBuf,Path};\n";
    }


    let code = format!("{}
use std::error::Error;
fn run() -> Result<(),Box<Error>> {{
{}
Ok(())
}}
fn main() {{
    run().unwrap();
}}
    ",prefix,code);

    let mut out_file = PathBuf::from(temp);
    out_file.push(&file);
    let mut program = out_file.clone();
    program.set_extension("");

    println!("***expanded code is in {}***",out_file.display());
    es::write_all(&out_file,code);

    process::Command::new("rustc")
        .args(&["-C","prefer-dynamic"])
        .args(&["-C","debuginfo=0"])
        //~ .args(&["--extern", "es=libes.so"])
        .arg("-o").arg(&program)
        .arg(&out_file)
        .status().or_die("can't run rustc");

    process::Command::new(&program)
        .env("LD_LIBRARY_PATH",format!("{}:.",rustup_lib()))
        .status()
        .or_die(&format!("can't run program {:?}",program));

}
