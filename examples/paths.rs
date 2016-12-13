extern crate easy_shortcuts as es;
use es::traits::*;

fn main() {
	let dir = es::argn_err(1,"please supply directory");
    for (p,m) in es::paths(&dir) {
        if m.is_file()  {
            println!("{:?} {}",p.file_name().unwrap(), m.len());
        }
    }
    // and now, loopless
    // filter is slightly awkward because we get references to the tuples
    es::paths(&dir)
		.filter(|&(_,ref m)| m.is_file())
		.map(|(p,m)| (p,m.len())).debug("\n");
}
