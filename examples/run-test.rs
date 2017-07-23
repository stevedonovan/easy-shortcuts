// a simple doc test runner.
// It is ASSUMED that you are in an immediate subdirectory of the crate
// root (I just use 'scratch') which can go into your .gitignore
// Packages up snippets using similar rules to cargo test.
extern crate easy_shortcuts as es;
use es::traits::*;
use std::env;
use std::fs;
use std::process;

fn get_crate_name() -> String {
    let mut crate_dir = env::current_dir().unwrap();
    crate_dir.pop();
    let crate_name = crate_dir.file_name().or_die("can't get crate");
    crate_name.to_str().unwrap().replace('-',"_").to_string()
}

fn append_indented(dest: &mut String, src: &str, indent: &str) {
    dest.extend(src.lines().map(|s| format!("{} {}\n",indent,s)));
}

fn main() {
    let file = es::argn_err(1,"please supply a source file");
    let kind = es::argn_or(2,"/");
    let crate_name = get_crate_name();
    let code = es::read_to_string(&file);
    let examples = "../examples";
    let (special,question) = kind.split_at(1);
    let indent = if special == "/" {"    "} else {""};
    let question = if question == "?" {true} else {false};

    let mut template = String::new();
    let mut before = String::new();
    let mut after = String::new();

    // Tests assume 'extern crate your_crate' unless there's already a declaration
    if code.find("extern crate").is_none() {
        template += &format!("extern crate {};\n",crate_name);
    }

    // they will also wrap your code in a main function
    if ! question {
        template += "fn main() {\n";
        template += &code;
        template += "}\n";
    } else {
        // unless you want to use the question-mark operator;
        // then we have to make up both a run() and main()
        before += "use std::error::Error;\n\n";
        before += "fn run() -> Result<(),Box<Error>> {\n";
        template += &before;
        template += &code;
        after += "Ok(())\n}\n\nfn main() {\n   run().unwrap();\n}";
        template += &after;
    }
    println!("{}",template);

    if ! fs::metadata(examples).is_dir() {
        fs::create_dir(examples).or_die("cannot create examples directory");
    }
    let test_file = format!("{}/{}",examples,file);
    es::write_all(&test_file,&template);

    // let cargo run the example
    let output = process::Command::new("cargo")
        .arg("run"). arg("-q")
        .arg("--color").arg("always") // let the Colours flow through, man
        .arg("--example")
        .arg(file.replace(".rs",""))
        .output().or_die("could not run cargo");

    println!("{}", String::from_utf8_lossy(&output.stderr));
    if output.stdout.len() > 0 {
        println!("WARNING - tests will suppress this output ******");
        println!("{}", String::from_utf8_lossy(&output.stdout));
    }
    if output.status.success() {
        println!("Copy and paste this output into your module ******\n");
        let comment = format!("{}//{}",indent,special);
        let guard = format!("{} ```\n",comment);
        let hide = format!("{} #",comment);
        let mut snippet = String::new();
        snippet.push_str(&guard);
        // before and after neeed '#' so they don't appear in the docs!
        append_indented(&mut snippet,&before,&hide);
        append_indented(&mut snippet,&code,&comment);
        append_indented(&mut snippet,&after,&hide);
        snippet.push_str(&guard);
        println!("{}",snippet);
    }
    fs::remove_file(&test_file).or_die("can't remove temporary file in examples");

}
