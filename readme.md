## Conveniences for Small Programs

Small programs which are meant to play nicely with other system commands
need to quit early and return a non-zero exit code. Panicking comes
across as untidy in such cases - it's a developer feature.

Consider a program that needs to read all of a file specified on the command line.

```rust
use std::io::prelude::*;
use std::fs::File;
use std::env;

let file = env::args().nth(1).expect("please supply a file name");

let mut f = File::open(&file).expect("can't open the file");
let mut s = String::new();
f.read_to_string(&mut s)).expect("can't read the string");
```
This is a little tedious if you _just want to quit_.

```rust
extern crate easy_shortcuts as es;

let file = es::argn_err(1,"please supply a file name");
let s = es::read_to_string(&file);
```
And passing a dud filename to this program gives us a clear fatal error:

```
./read_all error: open 'bonzo.txt' No such file or directory (os error 2)

```

Other than commands, there's another important category of small programs: little
bits of code you write when exploring an API. It's irritating to have to write
boilerplate when all you want to do is play with the contents of a file.

Another common need is to iterate over all the lines in a file:

```rust
extern crate easy_shortcuts as es:
use std::io;

for line in es::lines(io.stdin()) {
	// do your thang!
}
```

This creates the `io::BufReader` and cuts through the Java-esque ceremony to get
an iterator over all the lines of a file as strings; it will quit if there's an I/O
error while iterating.

## Iterator Shortcuts

Sometimes all you want to do is consume an iterator and print out its values. 
This little snippet will just echo the first ten lines of stdin to stdout:

```rust
use es::traits::*;

es::lines(io.stdin()).take(10).print("\n")
```

Here's a semi-useful example. Unix configuration files often have a large
amount of commented out options; this only prints out the options in force:

```rust
let conf = es::argn_err(1,"please provide a conf file");
es::lines(es::open(&conf))
    .filter(|s| s.len() > 0 && ! s.starts_with("#"))
    .print("\n");
```
And there's an equivalent `debug` which is extremely useful for finding out
what an iterator is actually pumping out:

```rust
(0..5).map(|n| (n,2*n)).debug("\n");
//->
(0, 0)
(1, 2)
(2, 4)
(3, 6)
(4, 8)
```

String methods like `skip_whitespace` return iterators, and easy_shortcuts 
presents some conveniences for processing and collecting strings:

```rust
let strs = ["one","two","three"];
let s = strs.into_iter().prepend(" hello ");
assert_eq!(s," hello one hello two hello three ");
```

And the old favourite of Pythonistas, `join`. (It's defined on vectors of
strings but as an iterator method we avoid unnecessary allocations.)

```rust
let s = "one two three".split_whitespace().join(',');
assert_eq!(s,"one,two,three");
```

## `collect` Considered Irritating

There are convenient `to_vec` and `to_map` methods available for iterators. `collect`
is very general, and its implementation is trivial: anything that implements
the `FromIterator` trait. Usually you have to give some type hints (the Rust 
compiler is not psychic yet) but nine times out of ten you want a `Vec`.

```rust
let v: Vec<_> = (0..5).collect();
// easier to read and totally equivalent
let v = (0..5).to_vec();
```

Here is a quick way to read a config file where keys and values are separated by
'=`'

```rust
	let map = es::lines(es::open(&conf))
      .filter(|s| ! s.starts_with("#")) // ignore commments
      .filter_map(|s| s.split_at_delim('=').trim()) // split into (String,String)
	  .to_map();
```


## Extra String Goodies

I could not resist adding a convenient new string method.

`split_at_delim` is like a combination of the string `find` and `split_at` methods,
except that the delimiter is not included; `"one = two".split_at_delim('=')` gives
`Some(("one "," two"))`. `trim` works on the result of this function, converting
`Option<(&str,&str)>` to `Option<(String,String)>` with any spare whitespace
trimmed.

In the above example, note that blank lines will be automatically ignored,
since `split_at_delim` will be `None`, which `trim` passes through.

Another convenience is `is_whitespace` defined on strings. This example counts source
lines and comment lines, assuming that comments are just '//'. 

```rust
extern crate easy_shortcuts as es;
use es::traits::*;

fn main() {
	let file = es::argn_err(1,"please provide a source file");
	let mut scount = 0;
	let mut ccount = 0;
	for line in es::lines(es::open(&file)) {
		if let Some(idx) = line.find("//") {
			// now look at everything up to //
			let start = &line[0..idx];
			if start.is_whitespace() {
				ccount += 1;
			}
		}
		scount += 1;	
	}
	println!("source lines {} comment lines {}",scount-ccount,ccount);
}
```

The preferred solution would be to use a regexp, but Rust's string handling
is pretty good on its own - string slices are a lovely feature. 

Functions that work on `Option<T>` or `Result<T,E>` are very convenient. For
instance, the `MetadataLike` trait adds the boolean methods of `fs::Metadata`. The 
reasoning is that usually you'd like to know if a dir entry exists _and_ it is of
the desired type.

```rust
let ok = fs::metadata(".").is_dir();

// which is short for:
let ok = match fs::metadata(".") {
	Ok(m) => m.is_dir(),
	Err(e) => false
}
```

## Directory Traversal

It is common to need to look at directory contents - the existing API is
a little clumsy, especially if you have a "quit early" policy in place. `paths`
provides an iterator over a directory giving a tuple of the path and associated
metadata.

```rust
extern crate easy_shortcuts as es;

fn has_extension(p: &std::path::Path, e: &str) -> bool {
	match p.extension() {
		Some(ext) => ext == e,
		None => false
	}
}

fn main() {
	for (path,data) in es::paths(".") {
		if data.is_file() && has_extension(&path,"rs") {
			println!("file {:?} len {}",path,data.len());
		}
	}
}

```

## Examples

`run-test.rs` is a small but non-trivial program using this crate which I wrote
to help me write doc tests.  Because (to be honest) it's an annoying process; you 
put code in _comments_, losing all those lovely visual clues from syntax highlighting,
and testing involves a full crate compile plus all those little crates generated
by the doc tests. With this little tool I could go from 20s to 0.5s writing doc 
snippets.

For this workflow, create a subdirectory in the crate directory (I just call it `scratch`
and add to `.gitignore`). The source file follows the same rules as doc tests 
themselves, `extern crate THIS_CRATE` is prepended and a `main` function created. It
compiles and runs the massaged code by copying it to the `examples` directory and
invoking `cargo run --example` on it.

```rust
~/rust/easy-shortcuts/scratch$ cat one.rs
let s = easy_shortcuts::read_to_string("one.rs");
println!("{}",s);
~/rust/easy-shortcuts/scratch$ run-test one.rs

let s = easy_shortcuts::read_to_string("one.rs");
println!("{}",s);


/// ```
/// let s = easy_shortcuts::read_to_string("one.rs");
/// println!("{}",s);
/// ```
```

We then write out the snippet with the appropriate doc comments. If you run with
an extra '!' argument, then module doc comments are created (using `//!`).

The next useful little program is `crate.rs`: given a crate name, it will look
in Cargo's source cache and print out the source directory of the _highest_
version of that crate.

```
~/rust/easy-shortcuts/examples$ cargo run -q --example crate semver
/home/steve/.cargo/registry/src/github.com-1ecc6299db9ec823/semver-0.2.3

```

It would require modification to work on Windows, and if it were a 'real' program
it would bring in dependencies on semver, glob, and maybe regex. But it is not meant
as an example of good Rust application style, but an example of using `easy_shortcuts`.




