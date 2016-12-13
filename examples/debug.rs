// debug() as a convenient consumer of an iterator
extern crate easy_shortcuts as es;
use es::traits::*;

fn main() {
	(0..5).map(|n| (n,2*n)).debug("\n");
}
