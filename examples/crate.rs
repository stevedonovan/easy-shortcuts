// Find Cargo's source cache directory for a crate
extern crate easy_shortcuts as es;
use es::traits::*;
use std::env;
use std::path;

fn path_file_name(p: &path::Path) -> String {
    p.file_name().unwrap().to_string_lossy().to_string()
}

// there's a Crate for This...
fn semver_i (s: &str) -> u64 {
    let v = s.split('.').filter_map(|s| s.parse::<u64>().ok()).to_vec();
    (((v[0] << 8) + v[1]) << 8) + v[2]
}

fn main() {
    let crate_name = es::argn_err(1,"please supply crate name");
    let home = env::var("CARGO_HOME") // set in cargo runs
        .unwrap_or(env::var("HOME").or_die("no home!") + "/.cargo");
    let crate_root = path::PathBuf::from(home + "/registry/src");
    // actual crate source is in some fairly arbitrary subdirectory of this
    let mut crate_dir = crate_root.clone();
    crate_dir.push(es::files(&crate_root).next().or_die("no crate cache directory"));    
    
    let mut crates = Vec::new();
    for (p,d) in es::paths(&crate_dir) {
        if ! d.is_dir() { continue; }
        let filename = path_file_name(&p);
        if let Some(endc) = filename.rfind('-') {
            if &filename[0..endc] == crate_name {
                crates.push((p,semver_i(&filename[endc+1..])));
            }
        }
    }
    // crate versions in ascending order by semver rules    
    crates.sort_by(|a,b| a.1.cmp(&b.1));
    let path = crates.last().or_die("no such crate");
    println!("{}",path.0.display());    
}
