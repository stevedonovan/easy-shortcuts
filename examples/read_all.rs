extern crate easy_shortcuts as es;

fn main() {
	let me = es::argn_or(1,"examples/read_all.rs");
	let text = es::read_to_string(&me);
	println!("{}",text);
}
