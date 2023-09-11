extern crate proc_macro_lib;
extern crate decl_macro_lib;
use proc_macro_lib::load_spec_file;
use proc_macro_lib::load_headers;

use decl_macro_lib::*;

#[allow(unused_imports)] use std::fs::File;
#[allow(unused_imports)] use std::io::{self, prelude::*, BufReader};

pub static __KMAG:  &str = "\x1B[35m";
pub static __KRED:  &str = "\x1B[31m";
pub static __KBLUE: &str = "\x1B[34m";
pub static __KNRM:  &str = "\x1B[0m";
pub static __KGRN:  &str = "\x1B[32m";
pub static __KCYN:  &str = "\x1B[36m";


fn main() -> std::io::Result<()> {
    load_headers!();
    load_spec_file!();
    
    {
        munch! {
            BASIC() {
                U32 x = MATCH(0xfefefefe); 
                U32 y {
                    0;12 => 0x1;
                    13;25 => 0xfea;
                    ENCODE(26;31 => id);
                };
            };
        }
    }
    Ok(())
}
