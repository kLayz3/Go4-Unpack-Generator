#[macro_export]
macro_rules! __hashmap {
    ( $($tag:ident = $val:expr ),* ) => {
        let hmap = HashMap::new();
        $(
            hmap.insert(stringify!($tag), stringify!($val).parse::<i32>().expect(&format!("{} cannot be parsed to int. Panicking...", stringify!($val))));
        )*
        hmap
    };
}
