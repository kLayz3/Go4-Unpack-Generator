extern crate proc_macro_lib;
extern crate decl_macro_lib;
use proc_macro_lib::load_spec_file;
use decl_macro_lib::*;

#[allow(unused_imports)] use std::fs::File;
#[allow(unused_imports)] use std::io::{self, prelude::*, BufReader};
#[allow(unused_imports)] use std::{thread, time::Duration};

static __KMAG:  &str = "\x1B[35m";
static __KRED:  &str = "\x1B[31m";
static __KBLUE: &str = "\x1B[34m";
static __KNRM:  &str = "\x1B[0m";
static __KGRN:  &str = "\x1B[32m";
static __KCYN:  &str = "\x1B[36m";

fn main() -> std::io::Result<()> {
    load_spec_file!();
    Ok(())
}
