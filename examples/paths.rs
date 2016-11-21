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
    es::paths(&dir)
		.filter(|item| item.1.is_file())
		.map(|item| (item.0,item.1.len())).debug("\n");
}
