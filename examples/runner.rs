// a simple Rust 'script' runner.
extern crate easy_shortcuts as es;
use es::traits::*;
use std::process;
use std::env;

fn rustup_lib() -> String {
    let tchain = es::shell("rustup toolchain list").lines()
        .find(|s| s.ends_with("(default)"))
        .or_die("no toolchain").to_string();
    let toolchain =  tchain.split_at_delim(' ').unwrap().0;
    let home = env::home_dir().unwrap();    
    format!("{}/.rustup/toolchains/{}/lib",home.display(),toolchain)
}

fn main() {
    let file = es::argn_err(1,"please supply a source file");
    let exti = file.find('.').or_die("must have an extension");
    if &file[exti..] != ".rs" {
        es::quit("must have extension .rs");
    }
    let code = es::read_to_string(&file);
    let mut prefix = String::new();
    if code.find("File::").is_some() {
        prefix += "use std::fs::File;\nuse std::io::prelude::*;\n";
    }
    if code.find("env::").is_some() {
        prefix += "use std::env;\n";
    }
    //println!("{}",rustup_lib());
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
    
    let out_file = format!("_{}",file);
    println!("***expanded code is in {}***",out_file);
    es::write_all(&out_file,code);
    process::Command::new("rustc")
        .args(&["-C","prefer-dynamic"])
        .args(&["-C","debuginfo=0"])
        .arg(&out_file)
        .status().or_die("can't invoke rustc");
    let program = &out_file[0..exti+1];
    process::Command::new(&format!("./{}",program))
        .env("LD_LIBRARY_PATH",&rustup_lib())
        .status()
        .or_die(&format!("can't run program {:?}",program));  
    
}
