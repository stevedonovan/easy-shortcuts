// rather like a simplified version of a 'head' utility!
extern crate easy_shortcuts as es;
use es::traits::*;

fn main() {
	let me = es::argn_or(1,"examples/lines.rs");	
	// es:lines() gives us a straight iterator over String
	// from a readable value;
	// The Print extension trait completes the one-liner.
	es::lines(es::open(&me)).take(3).print("\n");
}
