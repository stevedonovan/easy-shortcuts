use std::process::Command;
use std::io;
use std::fs::File;
use std::io::prelude::*;
use std::fmt::{Display,Debug};
use std::collections::HashMap;

pub mod traits {
	use std::collections::HashMap;

	pub trait ToVec<T> {
		fn to_vec(self) -> Vec<T>;
	}
	
	pub trait ToMap<K,V> {
		fn to_map(self) -> HashMap<K,V>;
	}

	/// this complements `Vec::join` on iterators
	/// and also for prepending text before each
	/// element
	pub trait Join {		
		fn join(self, delim: char) -> String;		
		fn prepend(self, prefix: &str) -> String;
		fn append<T: Fn(&str)->String>(self, map: T)->String;
	}

	/// provides a `print` method (over `Display`)
	/// to stdout with end char.
	pub trait Print {
		fn print(self,delim: &str);
	}

	/// provides a `debug` method (over `Debug`)
	/// to stdout with end char.
	pub trait PrintDbg {
		fn debug(self,delim: &str);
	}

	pub trait Die<T> {
		fn or_die(self, msg: &str) -> T;
	}
	
	pub trait StringEx {
		fn split_at_delim(&self, delim: char) -> Option<(&str,&str)>;
		fn split_at_delim_left(&self, delim: char) -> Option<(&str,&str)>;
		fn is_whitespace(&self) -> bool;
	}

}

use traits::*;
use std::path;
use std::fs;

impl <T> StringEx for T where T: AsRef<str> {
	/// splits the string into two parts; the part before
	/// the delimiter and the part after the delimiter.
	///
	/// ## Example
	///
	/// ```
	///     use easy_shortcuts::traits::StringEx;
	///
	/// 	let text = "one: two three";
	///     let res = text.split_at_delim(':');
	///     assert_eq!(res,Some(("one"," two three")));
	/// ```	
	fn split_at_delim(&self, delim: char) -> Option<(&str,&str)> {
		let s = self.as_ref();
		if let Some(idx) = s.find(delim) {			
			Some((&s[0..idx],&s[idx+1..]))
		} else {
			None
		}
	}
	
	fn split_at_delim_left(&self, delim: char) -> Option<(&str,&str)> {
		let s = self.as_ref();
		if let Some(idx) = s.rfind(delim) {			
			Some((&s[0..idx],&s[idx+1..]))
		} else {
			None
		}
	}	
	
	/// does this string only contain whitespace?
	fn is_whitespace(&self) -> bool {
		let s = self.as_ref();
		s.matches(char::is_whitespace).count() == s.len()
	}	
}

/*

impl <I,F> DirIerEx<iter::Map<I,F>> for  I
where I: Iterator<Item=PathPair>,
	  F: FnMut(PathPair) -> String
 {
	
	fn file_names(self) -> iter::Map<I,F> {
		self.map(|e: PathPair| e.0.file_name().unwrap().to_str().unwrap().to_string())
	}
	
}
*/

impl <T,E> Die<T> for Result<T,E>
where E: Display {
	/// this is like `expect` but quits with non-zero code
	/// instead of panicking	
	fn or_die(self, msg: &str) -> T {
		match self {
			Ok(t) => t,
			Err(e) => quit(&format!("{} {}",msg,e))
		}
	}	
}

impl <T,I> ToVec<T> for I
where T: Sized, I: Iterator<Item=T> {
	/// a more definite alternative to `collect`
	/// which collects an iterator's values into a Vec
	///
	/// ## Example
	/// 
	/// ```
	/// use easy_shortcuts::traits::ToVec;
	///
	/// let v = "one two three".split_whitespace().to_vec();
	/// assert_eq!(v,&["one","two","three"]);		
	/// ```	
    fn to_vec(self) -> Vec<T> {
        let mut res = Vec::new();
        for v in self {
            res.push(v);
        }
        res
    }
}

use std::cmp::Eq;
use std::hash::Hash;

impl <K,V,I> ToMap<K,V> for I
where K: Eq + Hash, V: Sized, I: Iterator<Item=(K,V)>   {
	fn to_map(self) -> HashMap<K,V> {
		let mut res = HashMap::new();
		for (k,v) in self {
			res.insert(k,v);
		}
		res
	}
	
}

// joining an iterator of strings
impl <T,I> Join for I
where T: AsRef<str>, I: Iterator<Item=T> {
	/// Join an iterator of strings using a delimiter.
	///
	/// ## Example
	///
	/// ```
	/// use easy_shortcuts::traits::Join;
	///
	/// let v = ["one","two","three"];
	/// let s = v.into_iter().map(|s| s.to_uppercase()).join(',');
	/// assert_eq!(s,"ONE,TWO,THREE");	
	/// ```	
    fn join(self, delim: char) -> String {
        let mut res = String::new();
        for v in self {
            res.push_str(v.as_ref());
            res.push(delim);
        }
        res.pop();
        res        
    }
    
	/// Join an iterator of strings by prepending a prefix
	///
	/// ```
	/// use easy_shortcuts::traits::Join;
	///
	/// let s = ["one","two","three"].into_iter().prepend(" -L");
	/// assert_eq!(s," -Lone -Ltwo -Lthree");	
	/// ```    
	fn prepend(self, prefix: &str) -> String  {
        let mut res = String::new();
        for v in self {
            res.push_str(&format!("{}{}",prefix,v.as_ref()));
        }
        res
	}
	
	fn append<F: Fn(&str)->String>(self, map: F)->String {
		let mut res = String::new();
		for s in self {
			res.push_str(&map(s.as_ref()));
		}
		res
	}
}


impl <T,I> Print for I
where T: Display, I: Iterator<Item=T> {
	///  Consume an iterator of values that implement Display
	///  and print them out to stdout with the given terminator.
	/// 
	/// ```
	///  use easy_shortcuts::traits::Print;
	/// 	
	///  [10,20,30].into_iter().print("\n");
	/// ```		
	fn print(self,delim: &str) {
		for v in self {
			print!("{}{}",v,delim);
		}
	}
}

impl <T,I> PrintDbg for I
where T: Debug, I: Iterator<Item=T> {
	fn debug(self,delim: &str) {
		for v in self {
			print!("{:?}{}",v,delim);
		}
	}
}


/// quit this program, printing a message and returning a non-zero exit code.
pub fn quit(msg: &str) -> !{
    writeln!(&mut io::stderr(),"{} error: {}",argn_or(0,""),msg).unwrap();
    std::process::exit(1);
}

/// a form of `quit` which works with the standard `Error` type.
pub fn quit_err(e: &std::error::Error) -> ! {
    quit(e.description());
}

macro_rules! quit {
    ($e:expr) => (match $e { Ok(val) => val, Err(err) => quit_err(&err) });
}

/// get the nth command-line argument or return the default.
pub fn argn_or(idx: usize, def: &str) -> String {
    std::env::args().nth(idx).unwrap_or(def.to_string())
}

/// get the nth argument or quit with a message.
pub fn argn_err(idx: usize, msg: &str) -> String {
    match std::env::args().nth(idx) {
    Some(s) => s,
    None => quit(&format!("no argument {}: {}",idx,msg))        
    }
}

/// open a file for reading, quitting if there's any error
pub fn open(file: &str) -> File {
    let f = File::open(file);
    if let Err(e) = f { 
        quit(&format!("open '{}' {}",file,e));
    }    
    f.unwrap()
}

/// create a file for writing, quitting if not possible
pub fn create(file: &str) -> File {
    let f = File::create(file);
    if let Err(e) = f { 
        quit(&format!("create '{}' {}",file,e));
    }    
    f.unwrap()    
}

/// read the contents of a file as a string, quitting otherwise
pub fn read_to_string(file: &str) -> String {
    let mut f = open(file);
    let mut s = String::new();
    quit!(f.read_to_string(&mut s));
    s
}

/// write bytes to a new file, or quit
pub fn write_all(file: &str, buff: &[u8]) {
    quit!(create(file).write_all(buff));
}

//pub fn exec(cmd: &str) {
	
//}

/// execute a shell command, combining stdout and stderr,
/// and return the result as a string
pub fn shell(cmd: &str) -> String {
    let o = Command::new("sh")
     .arg("-c")
     .arg(&format!("{} 2>&1",cmd))
     .output()
     .expect("failed to execute shell");
    String::from_utf8(o.stdout).unwrap().trim_right_matches('\n').to_string()
}

pub struct LineIter<R: io::Read> {
    iter: io::Lines<io::BufReader<R>>
}

/// iterate over all lines from a readable.
/// The iterator is over String (will quit if
/// there is an i/o error)
pub fn lines<R: io::Read>(f: R) -> LineIter<R> {
    LineIter{iter: io::BufReader::new(f).lines()}
}

impl <R: io::Read> Iterator for LineIter<R> {
    type Item = String;

    fn next(&mut self) -> Option<String> {
        let nxt = self.iter.next();
        if nxt.is_none() { return None; }
        Some(quit!(nxt.unwrap()))
    }    
}

pub struct DirIter {
    iter: std::fs::ReadDir
}

pub struct FileNameIter {
    iter: std::fs::ReadDir
}

impl Iterator for DirIter {
    type Item = (path::PathBuf, fs::Metadata);
    
    fn next(&mut self) -> Option<Self::Item> {
        let nxt = self.iter.next();
        if nxt.is_none() { return None; }
        let entry = quit!(nxt.unwrap());
        let meta = quit!(entry.metadata());
        Some((entry.path(),meta))
    }
}

impl Iterator for FileNameIter {
	type Item = String;
	
	fn next(&mut self) -> Option<Self::Item> {
        let nxt = self.iter.next();
        if nxt.is_none() { return None; }
        let file = quit!(nxt.unwrap()).file_name();
		Some(file.to_str().unwrap().to_string())		
	}
	
}

/// iterator over all entries in a directory;
/// Returns a tuple of (`path::PathBuf`,`fs::Metadata`)
/// will quit if the directory does not exist or there
/// is an i/o error)
pub fn paths (dir: &str) -> DirIter {
    let res = std::fs::read_dir(dir);
    match res {
        Ok(s) => DirIter{iter: s},
        Err(e) => quit(&format!("'{}' {}",dir,e))
    }    
}

/// iterator over all files in a directory;
/// Returns the files as strings
/// will quit if the directory does not exist or there
/// is an i/o error)
pub fn files (dir: &str) -> FileNameIter {
    let res = std::fs::read_dir(dir);
    match res {
        Ok(s) => FileNameIter{iter: s},
        Err(e) => quit(&format!("'{}' {}",dir,e))
    }    
}


#[cfg(test)]
mod tests {
    use traits::*;

	
    #[test]
    fn test_to_vec() {
		let v = "one two three".split_whitespace().to_vec();
		assert_eq!(v,&["one","two","three"]);		
	}
	
	#[test]
	fn test_iter_join() {
		// join an iterator over &str
		let v = ["one","two","three"];
		let s = v.into_iter().join(',');
		assert_eq!(s,"one,two,three");
		
		// join an iterator over String
		let v = vec!["one".to_string(),"two".to_string(),"three".to_string()];
		let s = v.into_iter().map(|s| s.to_uppercase()).join(',');
		assert_eq!(s,"ONE,TWO,THREE");	
		
		let s = ["one","two","three"].into_iter().prepend(" -L");
		assert_eq!(s," -Lone -Ltwo -Lthree");	
	}
}    

