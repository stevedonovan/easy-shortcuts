extern crate easy_shortcuts as es;
//~ use es::traits::*;
use std::time::Duration;

fn main() {
    let dir = es::argn_or(1,".");
    // .show_all().name(".git")
    // .extension("rs")
    let d = Duration::from_secs(60);
    //~ for (p,_) in es::all_paths(&dir).files_only().modified_since(d) {
    for (p,_) in es::all_paths(&dir).exclude("target") {
        println!("{:?}",p);
    }

}
