// Find Cargo's source cache directory for a crate
extern crate easy_shortcuts as es;
use es::traits::*;
use std::env;
use std::path;

fn path_file_name(p: &path::Path) -> String {
    p.file_name().unwrap().to_string_lossy().to_string()
}

fn char_at(s: &str, i: usize) -> char {
    s.chars().nth(i).unwrap()
}

// there's a Crate for This...
fn semver_i (s: &str) -> u64 {
    let v = s.split('.').filter_map(|s| s.parse::<u64>().ok()).to_vec();
    (((v[0] << 8) + v[1]) << 8) + v[2]
}

fn main() {
    let mut crate_name = es::argn_err(1,"please supply crate name");
    let home = env::var("CARGO_HOME").unwrap_or(env::var("HOME").or_die("no home!") + "/.cargo");
    // Use the Shell, Luke!
    println!("{:?}",home);
    let crate_dir = es::shell(&format!("echo {}/registry/src/*",home));
    if crate_dir.find('*').is_some() {
        es::quit("no cargo cache");
    }
    crate_name += "-";
    let endc = crate_name.len();
    
    let mut crates = Vec::new();
    for (p,d) in es::paths(&crate_dir) {
        if ! d.is_dir() { continue; }
        let filename = path_file_name(&p);
        // well yes, regex would be rather more elegant here ;)
        // This distinguishes between 'regex' and 'regex-syntax' etc
        if filename.starts_with(&crate_name) && char_at(&filename,endc).is_digit(10) {
            crates.push((p,semver_i(&filename[endc..])));
        }
    }
    // crate versions in ascending order by semver rules    
    crates.sort_by(|a,b| a.1.cmp(&b.1));
    let path = crates.last().or_die("no such crate");
    println!("{}",path.0.display());    
}
