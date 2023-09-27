extern crate proc_macro;

use proc_macro::TokenStream;
use regex::Regex;
use std::process::Command;

macro_rules! exec_command {
    ($com_name: ident $($arg:tt)*) => {
        String::from_utf8(
            Command::new(stringify!($com_name))
            $(
            .arg(stringify!($arg))
            )*
            .output()
            .unwrap()
            .stdout).unwrap()
    };
    () => {};
}

#[proc_macro]
pub fn load_spec_file(_item: TokenStream) -> TokenStream {
    let s =  std::fs::read_to_string("event.spec").unwrap();
    println!("Parsing structure: {}", s);
    let mut head: TokenStream = "munch!".parse().unwrap();
    let tail: String = "(".to_owned() + &s + ")";
    head.extend(tail.parse::<TokenStream>().unwrap());
    head
}
#[proc_macro]
pub fn load_headers(_item: TokenStream) -> TokenStream {
    let mut head: TokenStream = "munch!()".parse().unwrap();

    let re = Regex::new(r##"^#include[ \t\n]+"([\w\-/\.]+)"$"##).unwrap(); 
    let s = std::fs::read_to_string("event.spec").unwrap();
    
    let mut pwd = Command::new("pwd");
    let mut curr_dir = exec_command!(pwd);

    let pwd_dir: String = String::from_utf8(Command::new("ls").arg("../").output().unwrap().stdout).unwrap(); 

    println!("Curr_dir : {}", curr_dir);
    println!("With PWD: {}", pwd_dir);

    pwd.current_dir("../");
    curr_dir = get_result(&mut pwd);
    println!("Curr pwd dir : {}", curr_dir); 

    // Find all occurences of `include` in the file:
    // Do this iteratively. In case includes have other includes,
    // until all `include` are resolved, while keeping track of possible
    // cross referencing. Perform DFS and parse the terminal nodes
    // first. Include files should always be in /common or .. or ../common
    let mut it = re.captures_iter(&s);
    let mut files_seen: Vec<String> = vec![];
    while let Some(caps) = it.next() {
        files_seen.push(caps[1].to_owned());
        // Process includes inside the capture:
        process_file(&caps[1], &mut pwd, &mut files_seen);
    }

    head
}


#[inline]
fn get_result(command: &mut Command) -> String {
    String::from_utf8(command.output().unwrap().stdout).unwrap()
}
#[allow(unused_variables)]
fn process_file(file_name: &str, file_dir: &mut Command, f_stack: &mut Vec<String>) {

}
