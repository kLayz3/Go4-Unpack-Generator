#[macro_export]
macro_rules! parse_generics {
    ( $($generic:ident),* ) => {{
        let mut __s = String::new();
        $( __s += &format!("uint32_t {}, ", stringify!($generic)); )* 
        __s
    }};
}
