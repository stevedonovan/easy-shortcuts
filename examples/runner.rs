// a simple Rust 'script' runner.
extern crate easy_shortcuts as es;
use es::traits::*;
use std::process;
use std::env;
use std::fs;
use std::path::PathBuf;

fn rustup_lib() -> String {
    es::shell("rustc --print sysroot") + "/lib"
}

fn indent_line(line: &str) -> String {
    format!("    {}\n",line)
}

const PRELUDE: &'static str = "
#![allow(unused_imports)]
#![allow(dead_code)]
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::env;
use std::path::{PathBuf,Path};

macro_rules! debug {
    ($x:expr) => {
        println!(\"{} = {:?}\",stringify!($x),$x);
    }
}

";

fn prelude_and_cache() -> (String,PathBuf) {
    let mut home = env::home_dir().or_die("no home!");
    home.push(".cargo");
    home.push(".runner");
    let pristine = ! home.is_dir();
    if pristine {
        fs::create_dir(&home).or_die("cannot create runner directory");
    }
    let mut prelude = home.clone();
    prelude.push("prelude");
    home.push("dy-cache");
    if pristine {
        es::write_all(&prelude,PRELUDE);
        fs::create_dir(&home).or_die("cannot create dynamic cache");
    }
    (es::read_to_string(&prelude),home)
}

fn main() {
    // we are going to put the expanded source and resulting exe in temp
    let out_dir = "temp";
    if ! fs::metadata(out_dir).is_dir() {
        fs::create_dir(out_dir).or_die("cannot create temp directory here");
    }

    let file = PathBuf::from(es::argn_err(1,"please supply a source file"));
    let ext = file.extension().or_die("no file extension");
    if ext != "rs" {
        es::quit("file extension must be .rs");
    }

    // we'll pass rest of arguments to program
    let args = env::args().skip(2).to_vec();


    let mut code = es::read_to_string(&file);

    let (prelude,cache) = prelude_and_cache();

    if code.find("fn main").is_none() {
        let mut prefix = prelude;
        let mut body = String::new();
        {
            let mut lines = code.lines();
            for line in lines.by_ref() {
                let line = line.trim_left();
                if line.starts_with("//") || line.len() == 0 ||
                    line.starts_with("extern ") || line.starts_with("use ") {
                    prefix += line;
                    prefix.push('\n');
                } else {
                    body += &indent_line(line);
                    break;
                }
            }
            // and indent the rest!
            body.extend(lines.map(indent_line));
        }

        code = format!("{}
    use std::error::Error;
    fn run() -> Result<(),Box<Error>> {{
    {}    Ok(())
    }}
    fn main() {{
        run().unwrap();
    }}
        ",prefix,body);
    }

    let mut out_file = PathBuf::from(out_dir);
    out_file.push(&file);
    let mut program = out_file.clone();
    program.set_extension("");

    es::write_all(&out_file,&code);

    let mut builder = process::Command::new("rustc");
    builder.args(&["-C","prefer-dynamic"]).args(&["-C","debuginfo=0"])
           .arg("-L").arg(&cache);
    let status = builder.arg("-o").arg(&program)
        .arg(&out_file)
        .status().or_die("can't run rustc");
    if ! status.success() {
        return;
    }

    process::Command::new(&program)
        .env("LD_LIBRARY_PATH",format!("{}:{}",rustup_lib(),cache.display()))
        .args(&args)
        .status()
        .or_die(&format!("can't run program {:?}",program));

}
