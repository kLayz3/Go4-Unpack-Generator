extern crate proc_macro;
use proc_macro::TokenStream;

#[proc_macro]
pub fn make_answer(_item: TokenStream) -> TokenStream {
    "fn answer() -> u32 { 42 }".parse().unwrap()
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
    let s = std::fs::read_to_string("event.spec").unwrap();
    let re = Regex::new(r##"^#include[ \t]+"([\w/\.]+)"$"##); 
}

#[proc_macro]
pub fn proc_test_lex(_item: TokenStream) -> TokenStream {
    let mut head = "test_lex!".parse::<TokenStream>().unwrap(); 
    let end = "(kekega)".parse::<TokenStream>().unwrap();

    head.extend(end);
    head
}
