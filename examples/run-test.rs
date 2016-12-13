// a simple doc test runner.
// It is ASSUMED that you are in an immediate subdirectory of the crate
// root (I just use 'scratch') which can go into your .gitignore
// Packages up snippets using similar rules to the cargo test running.
extern crate easy_shortcuts as es;
use es::traits::*;
use std::env;
use std::fs;
use std::process;

fn get_crate_name() -> String {
    let mut crate_dir = env::current_dir().unwrap();
    crate_dir.pop();
    let crate_name = crate_dir.file_name().or_die("can't get crate");
    crate_name.to_str().unwrap().replace('-',"_").to_string()
}

fn main() {
    let file = es::argn_err(1,"please supply a source file");
    let special = es::argn_or(2,"/");
    let crate_name = get_crate_name();
    let code = es::read_to_string(&file);
    let examples = "../examples";

    // massage the snippet into a proper Rust program in examples dir
    let template = {
        // we have to add a main function....
        if code.find("fn main").is_none() {
            // extern crate provided already (mebbe with alias?)
            let this_crate =
                if code.starts_with("extern crate") {"".to_string()}
                else {format!("extern crate {};",crate_name)};
            format!("{}\nfn main() {{\n{}\n}}\n",this_crate,code)
        } else {
            code.clone()
        }
    };

    if ! fs::metadata(examples).is_dir() {
        fs::create_dir(examples).or_die("cannot create examples directory");
    }
    let test_file = format!("{}/{}",examples,file);
    es::write_all(&test_file,template);

    // let cargo do its magic
    let output = process::Command::new("cargo")
        .arg("run"). arg("-q")
        .arg("--color").arg("always") // let the Colours flow through, man
        .arg("--example")
        .arg(file.replace(".rs",""))
        .output().or_die("could not run cargo");

    println!("{}", String::from_utf8_lossy(&output.stderr));
    println!("{}", String::from_utf8_lossy(&output.stdout));
    if output.status.success() {
        let comment = format!("//{}",special);
        let guard = format!("{} ```\n",comment);
        let mut snippet = guard.clone();
        snippet.extend(code.lines().map(|s| format!("{} {}\n",comment,s)));
        snippet.push_str(&guard);
        println!("{}",snippet);
    }
    fs::remove_file(&test_file).or_die("can't remove temporary file in examples");

}
