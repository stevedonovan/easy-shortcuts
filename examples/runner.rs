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

fn is_iden_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn names_after(mut text: &str, target: &str) -> Vec<String> {
    let mut res = Vec::new();
    while let Some(idx) = text.find(target) {
        text = &text[idx+target.len()..];
        let ns = text.find(is_iden_char).or_die("no iden following");
        text = &text[ns..];
        let ne = text.find(|c: char| ! is_iden_char(c)).or_die("no iden finish");
        res.push((&text[0..ne]).to_string());
    }
    res
}

fn indent(line: &str) -> String {
    format!("    {}\n",line)
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
    
    // we'll pass rest of arguments to program
    let args = env::args().skip(2).to_vec();

    let code = es::read_to_string(&file);

    let mut prefix = String::new();
    if found(&code,"File::") {
        prefix += "use std::fs::File;\nuse std::io::prelude::*;\n";
    }
    if found(&code,"env::") {
        prefix += "use std::env;\n";
    }
    if found(&code,"PathBuf::") {
        prefix += "use std::path:{PathBuf,Path};\n";
    }
    
    let mut lines = code.lines();
    let mut body = String::new();
    for line in lines.by_ref() {
        let line = line.trim_left();
        if line.starts_with("//") || line.len() == 0 ||
            line.starts_with("extern ") || line.starts_with("use ") {
            prefix += line;
            prefix.push('\n');
        } else {
            body += &indent(line);
            break;
        }
    }
    // and indent the rest!
    body.extend(lines.map(indent_line));
    
    let crates = names_after(&prefix,"extern crate");

    let code = format!("{}
use std::error::Error;
fn run() -> Result<(),Box<Error>> {{
{}    Ok(())
}}
fn main() {{
    run().unwrap();
}}
    ",prefix,body);

    let mut out_file = PathBuf::from(temp);
    out_file.push(&file);
    let mut program = out_file.clone();
    program.set_extension("");

    println!("***expanded code is in {}***",out_file.display());
    es::write_all(&out_file,code);

    let mut builder = process::Command::new("rustc");
    builder.args(&["-C","prefer-dynamic"]).args(&["-C","debuginfo=0"]);
    for c in crates {
        let cref = format!("{}=lib{}.so",c,c);
        builder.args(&["--extern",&cref]);
    }
    builder.arg("-o").arg(&program)
        .arg(&out_file)
        .status().or_die("can't run rustc");

    process::Command::new(&program)
        .env("LD_LIBRARY_PATH",format!("{}:.",rustup_lib()))
        .args(&args)
        .status()
        .or_die(&format!("can't run program {:?}",program));

}
