//! Small programs which are meant to interoperate with other system commands
//! need to quit early and return a non-zero exit code. Panicking comes
//! across as unprofessional in such cases.
//! This crate provides some useful shortcuts if you are writing programs
//! that fit this 'fail hard and early' pattern.
//!
//! Errors go through the `quit` function. You can switch the behaviour back to panicking
//! with the EASY_DONT_QUIT_PANIC environment variable.
//!
//! Functions like `open` and `create` wrap the usual `io::File` methods,
//! except that they quit instead of panicking.  There are functions
//! `read_to_string` and `write_all` to read and write the contents of
//! text files, and `lines` provides a straight iterator over all lines
//! in a readable.  `paths` provides an iterator over `(PathBuf,Metadata)`
//! pairs in a directory
//!
//! The `or_die` method is implemented for option and result types, allowing
//! a Perl-style equivalent to the usual `expect`.
//!
//! `to_vec` and `to_map` are trivial specializations of `collect` in the
//! usual case where you do want these containers. `print` and `debug` are
//! convenient consumers of iterators which dump out `Display` or `Debug` values.
//!
//! Building up strings is a common operation; `join` is already defined on
//! vectors of strings, but here it's also defined on iterators of strings.
//! `prepend` collects strings with some text prepended to each value:
//!
//! ## Examples
//!
//! ```
//! use easy_shortcuts::traits::*;
//!
//! (0..).take(5).print("\n");
//!
//! (0..5).map(|n| (n,2*n)).debug("\n");
//!
//! let strs = ["one","two","three"];
//! let s = strs.into_iter().prepend(" -L");
//! assert_eq!(s," -Lone -Ltwo -Lthree");
//! ```

use std::io;
use std::fs::File;
use std::process::Command;
use std::io::prelude::*;
use std::fmt::{Display,Debug};
use std::collections::HashMap;


pub mod traits {
    use std::collections::HashMap;

    /// convenient to_vec() method on iterators
    pub trait ToVec<T> {
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
        fn to_vec(self) -> Vec<T>;
    }

    /// convenient to_map() method on iterators
    pub trait ToMap<K,V> {
        /// collect values into a HashMap
        fn to_map(self) -> HashMap<K,V>;
    }

    /// string collection methods on iterators
    pub trait Join {
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
        fn join(self, delim: char) -> String;

        /// Join an iterator of strings by prepending a prefix
        ///
        /// ```
        /// use easy_shortcuts::traits::Join;
        ///
        /// let s = ["one","two","three"].into_iter().prepend(" -L");
        /// assert_eq!(s," -Lone -Ltwo -Lthree");
        /// ```
        fn prepend(self, prefix: &str) -> String;

        /// Join strings by collecting the result of an arbitrary function
        fn append<T: Fn(&str)->String>(self, map: T)->String;
    }

    /// provides a print() method over iterators
    pub trait Print {
        ///  Consume values that implement Display
        ///  and print them out to stdout with the given terminator.
        ///
        /// ```
        ///  use easy_shortcuts::traits::Print;
        ///
        ///  [10,20,30].into_iter().print("\n");
        /// ```
        fn print(self,delim: &str);
    }

    /// provides a `debug` method on iterators
    pub trait PrintDbg {
        ///  Consume values that implement Debug
        ///  and print them out to stdout with the given terminator.
        ///
        /// ```
        ///  use easy_shortcuts::traits::PrintDbg;
        ///
        ///  (0..5).map(|n| (n,2*n)).debug("\n");
        /// ```
        fn debug(self,delim: &str);
    }

    /// Perl-like 'die' quit on error
    pub trait Die<T> {
        /// this is like `expect` but quits with non-zero code
        /// instead of panicking. Defined for Option<T>
        /// and Result<T,E>
        ///
        /// extern crate easy_shortcuts;
        /// use easy_shortcuts::traits::Die;
        /// use std::env;
        /// let = env::home_dir().or_die("no home!");
        fn or_die(self, msg: &str) -> T;
    }

    /// useful extra string operations
    pub trait StringEx {
        /// splits the string into two parts; the part before
        /// the delimiter and the part after the delimiter.
        ///
        /// ## Example
        ///
        /// ```
        ///     use easy_shortcuts::traits::StringEx;
        ///
        ///     let text = "one: two three";
        ///     let res = text.split_at_delim(':');
        ///     assert_eq!(res,Some(("one"," two three")));
        /// ```
        fn split_at_delim(&self, delim: char) -> Option<(&str,&str)>;

        /// splits the string into two parts; the part before
        /// the delimiter and the part after the delimiter.
        /// But this version searches from the right!
        ///
        /// ## Example
        ///
        /// ```
        ///     use easy_shortcuts::traits::StringEx;
        ///
        ///     let text = "one: two :three four";
        ///     let res = text.split_at_delim_right(':');
        ///     assert_eq!(res,Some(("one: two ","three four")));
        /// ```
        fn split_at_delim_right(&self, delim: char) -> Option<(&str,&str)>;

        /// does this string only contain whitespace?
        fn is_whitespace(&self) -> bool;
    }

    /// trims pairs of strings, passes through None
    pub trait MaybeTrim {
        /// this operates on pairs of strings that may be empty
        /// and creates a pair of trimmed strings if possible.
        ///
        /// It's designed to work with methods like `split_at_delim`
        ///
        /// ```
        /// use easy_shortcuts::traits::*;
        ///
        /// let s = " one = two ".split_at_delim('=').trim();
        /// assert_eq!(s,Some(("one".to_string(),"two".to_string())));
        /// ```
        fn trim(self) -> Option<(String,String)>;
    }


    /// file system queries. They are defined
    /// on `io.Result<fs::Metadata>`.
    pub trait MetadataLike {
        /// is this a directory?
        ///
        /// ```
        /// use easy_shortcuts::traits::MetadataLike;
        /// use std::fs;
        ///
        /// let res = fs::metadata(".").is_dir();
        /// assert!(res);
        /// ```
        fn is_dir(self) -> bool;

        /// is this a file?
        ///
        /// ```
        /// use easy_shortcuts::traits::MetadataLike;
        /// use std::fs;
        ///
        /// let res = fs::metadata("bonzo.dog").is_file();
        /// assert!(! res);
        /// ```
        fn is_file(self) -> bool;
    }

}

use traits::*;
use std::path;
use std::fs;

impl <T> StringEx for T where T: AsRef<str> {
    fn split_at_delim(&self, delim: char) -> Option<(&str,&str)> {
        let s = self.as_ref();
        if let Some(idx) = s.find(delim) {
            Some((&s[0..idx],&s[idx+1..]))
        } else {
            None
        }
    }

    fn split_at_delim_right(&self, delim: char) -> Option<(&str,&str)> {
        let s = self.as_ref();
        if let Some(idx) = s.rfind(delim) {
            Some((&s[0..idx],&s[idx+1..]))
        } else {
            None
        }
    }

    fn is_whitespace(&self) -> bool {
        let s = self.as_ref();
        s.matches(char::is_whitespace).count() == s.len()
    }
}

impl <'a>MaybeTrim for Option<(&'a str,&'a str)> {

    fn trim(self) -> Option<(String,String)> {
        match self {
            Some((s1,s2)) => Some((s1.trim().to_string(),s2.trim().to_string())),
            None => None
        }
    }

}


impl <T,E> Die<T> for Result<T,E>
where E: Display {
    fn or_die(self, msg: &str) -> T {
        match self {
            Ok(t) => t,
            Err(e) => quit(&format!("{} {}", msg,e))
        }
    }
}

impl <T> Die<T> for Option<T>  {
    fn or_die(self, msg: &str) -> T {
        match self {
            Some(t) => t,
            None => quit(msg)
        }
    }
}

use std::iter::FromIterator;

impl <T,I> ToVec<T> for I
where T: Sized, I: Iterator<Item=T> {
    fn to_vec(self) -> Vec<T> {
        FromIterator::from_iter(self)
    }
}

use std::cmp::Eq;
use std::hash::Hash;

impl <K,V,I> ToMap<K,V> for I
where K: Eq + Hash, V: Sized, I: Iterator<Item=(K,V)>   {
    fn to_map(self) -> HashMap<K,V> {
        FromIterator::from_iter(self)
    }
}

impl <T,I> Join for I
where T: AsRef<str>, I: Iterator<Item=T> {
    fn join(self, delim: char) -> String {
        let mut res = String::new();
        for v in self {
            res.push_str(v.as_ref());
            res.push(delim);
        }
        res.pop();
        res
    }

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

impl MetadataLike for io::Result<fs::Metadata> {
    fn is_dir(self) -> bool {
        match self {
        Ok(meta) => meta.is_dir(),
        Err(_) => false
        }
    }

    fn is_file(self) -> bool {
        match self {
        Ok(meta) => meta.is_file(),
        Err(_) => false
        }
    }
}


/// quit this program, printing a message and returning a non-zero exit code.
pub fn quit(msg: &str) -> !{
    let text = format!("{} error: {}",argn_or(0,""),msg);
    if std::env::var("EASY_DONT_QUIT_PANIC").is_ok() {
        panic!(text);
    } else {
        writeln!(&mut io::stderr(),"{}",text).unwrap();
        std::process::exit(1);
    }
}

/// a form of `quit` which works with the standard `Error` type.
pub fn quit_err(e: &std::error::Error) -> ! {
    quit(e.description());
}


/// quit works like try, except it quits instead of returning.
/// The error message is output using `quit_err`.
///
/// ```
/// #[macro_use]
/// extern crate easy_shortcuts;
/// use std::fs;
///
/// fn main() {
///     let md = quit!(fs::metadata("."));
/// }
/// ```
#[macro_export]
macro_rules! quit {
    ($e:expr) => (match $e { Ok(val) => val, Err(err) => $crate::quit_err(&err) });
}

/// get the nth command-line argument or return the default.
///
/// ```
/// extern crate easy_shortcuts as es;
///
/// let arg = es::argn_or(1,"default");
/// assert_eq!(arg,"default");
/// ```
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

use std::path::Path;

/// open a file for reading, quitting if there's any error.
pub fn open<P: AsRef<Path>>(file: P) -> File {
    match  File::open(&file) {
        Ok(f) => f,
        Err(e) => quit(&format!("open {:?} {}",file.as_ref(),e))
    }
}

/// create a file for writing, quitting if not possible.
pub fn create<P: AsRef<Path>>(file: P) -> File {
    match File::create(&file) {
        Ok(f) => f,
        Err(e) => quit(&format!("create {:?} {}",file.as_ref(),e))
    }
}

/// read the contents of a file as a string, quitting otherwise
pub fn read_to_string<P: AsRef<Path>>(file: P) -> String {
    let mut f = open(file);
    let mut s = String::new();
    quit!(f.read_to_string(&mut s));
    s
}

/// write a String to a new file, or quit
pub fn write_all<P: AsRef<Path>>(file: P, buff: String) {
    quit!(create(file).write_all(&buff.into_bytes()));
}

/// execute a shell command, combining stdout and stderr,
/// and return the result as a string
///
/// ```
/// extern crate easy_shortcuts as es;
///
/// let res = es::shell("rustc -V");
/// assert!(res.starts_with("rustc"));
/// ```
pub fn shell(cmd: &str) -> String {
    let o = Command::new("sh")
     .arg("-c")
     .arg(&format!("{} 2>&1",cmd))
     .output()
     .expect("failed to execute shell");
    quit!(String::from_utf8(o.stdout)).trim_right_matches('\n').to_string()
}

/// implements line iterator over a readable.
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
        match nxt {
            Some(p) => Some(quit!(p)),
            None => None
        }
    }
}

/// implements directory iterator over (path,metatable)
pub struct DirIter {
    iter: std::fs::ReadDir
}

/// implements directory iterator over filenames
pub struct FileNameIter {
    iter: std::fs::ReadDir
}

impl Iterator for DirIter {
    type Item = (path::PathBuf, fs::Metadata);

    fn next(&mut self) -> Option<Self::Item> {
        let nxt = self.iter.next();
        match nxt {
            None => None, // end of iterator
            Some(me) => {
                let entry = quit!(me);
                let meta = quit!(entry.metadata());
                Some((entry.path(),meta))
            }

        }
    }
}

impl Iterator for FileNameIter {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            None => None,
            Some(me) => {
                let entry = quit!(me);
                let file = entry.file_name();
                Some(file.to_string_lossy().to_string())
            }
        }
    }

}


/// iterator over all entries in a directory.
/// Returns a tuple of (`path::PathBuf`,`fs::Metadata`);
/// will quit if the directory does not exist or there
/// is an i/o error)
pub fn paths<P: AsRef<Path>> (dir: P) -> DirIter {
    match std::fs::read_dir(dir.as_ref()) {
        Ok(s) => DirIter{iter: s},
        Err(e) => quit(&format!("{:?} {}",dir.as_ref(),e))
    }
}

/// iterator over all files in a directory.
/// Returns the files as strings;
/// will quit if the directory does not exist or there
/// is an i/o error)
pub fn files<P: AsRef<Path>> (dir: P) -> FileNameIter {
    match std::fs::read_dir(dir.as_ref()) {
        Ok(s) => FileNameIter{iter: s},
        Err(e) => quit(&format!("{:?} {}",dir.as_ref(),e))
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

