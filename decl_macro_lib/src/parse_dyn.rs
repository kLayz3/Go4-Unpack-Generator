#[macro_export]
macro_rules! parse_dyn_decl {
    ($field_type:ident ( $($field_generic:expr),* ) $field_name:ident) => {{
        let mut __s = String::new();

        // Declaration and fields:
        __s += &formatt!(1; "struct {{\n");
        __s += &formatt!(2; "unsigned num_items = 0;\n");
        let mut field_type = format!("_{}<", stringify!($field_type));
        $( field_type += &format!("{},", stringify!($field_generic)); )*
        field_type += &format!("void>");
        __s += &formatt!(2; "{} data[_MAX_DYN];\n", field_type);
        __s
    }};
}

#[macro_export]
macro_rules! parse_dyn_common {
    // This macro will create the `fill` and `clear` methods of dyn fields
    ($field_type:ident $field_name:ident) => {{
        let mut __s = String::new();

         // `fill()`
        __s += &formatt!(2; "void fill(uint8_t* event_handle, size_t& bytes_available, size_t& bytes_read) {{\n");
        __s += &formatt!(3; "bytes_read = 0;\n");
        __s += &formatt!(3; "size_t bytes_read_sub;\n");
        __s += &formatt!(3; "while(bytes_available < _{}::min_size() && num_items < _MAX_DYN) {{\n", stringify!($field_type));
        __s += &formatt!(4; "data[num_items].fill(event_handle, bytes_available, bytes_read_sub);\n");
        __s += &formatt!(4; "if(!check_current()) {{\n");
        __s += &formatt!(5; "data[num_items].clear();\n");
        __s += &formatt!(5; "break;\n");
        __s += &formatt!(4; "}}\n");
        __s += &formatt!(4; "event_handle += bytes_read_sub;\n");
        __s += &formatt!(4; "bytes_read += bytes_read_sub;\n");
        __s += &formatt!(4; "num_items++;\n");
        __s += &formatt!(3; "}}\n");
        __s += &formatt!(2; "}}\n");

        // `clear()`
        __s += &formatt!(2; "inline void clear() noexcept {{\n");
        __s += &formatt!(3; "for(int _i = 0; _i < num_items; ++_i) data[_i].clear();\n");
        __s += &formatt!(2; "}}\n");
        __s
    }};
}
#[macro_export]
macro_rules! parse_dyn_init {
    // This will create the `init()` method, in case the dyn field has an ENCODE inside.
    // Only for primitive fields with a '{}' block!
    () => {{
        let mut __s = String::new();
        __s += &formatt!(2; "void init() {{\n");
        __s += &formatt!(3; "for(int _i = 0; _i < _MAX_DYN; ++_i) data[_i].init();\n");
        __s += &formatt!(2; "}}\n");
        __s
    }};
    ($field_type:ident $field_name:ident { $($condition_body:tt)* }) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "void init() {{\n");
        __s += &parse_dyn_init_inside!(@$($condition_body)*);
        __s += &formatt!(2; "}}\n");
        __s
    }};
}

#[macro_export]
macro_rules! parse_dyn_check { 
    ($field_type:ident $field_name:ident) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "inline bool check_current() {{\n");
        __s += &formatt!(3; "return data[num_items].check_event();\n");
        __s += &formatt!(2; "}}\n");
        __s
    }};
    ($field_type:ident $field_name:ident = MATCH($match_val:expr)) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "inline bool check_current() {{\n");
        __s += &formatt!(3; "return data[num_items] == {};\n", stringify!($match_val));
        __s += &formatt!(2; "}}\n");
        __s
    }};

    ($field_type:ident $field_name:ident { $($condition_body:tt)* } ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "bool check_current() {{\n");
        __s += &formatt!(3; "bool __b = 1;\n");
        __s += &parse_dyn_check_inside!(@$($condition_body)*);
        __s += &formatt!(2; "}}\n");
        __s 
    }};
}

#[macro_export]
macro_rules! parse_dyn_check_inside {
    (@ $left_bound:tt .. $right_bound:expr => $assert_val:expr ; $($rest:tt)*) => {{
        let mut __s = String::new();
        __s += &formatt!(3; "{{\n");
        __s += &formatt!(4; "uint32_t __mask = (uint32_t)((1ull << ({} - ({}))) - 1);\n", stringify!($right_bound), stringify!($left_bound));
        __s += &formatt!(4; "uint32_t __word = (uint32_t)(data[num_items] >> ({}));", stringify!($left_bound));
        __s += &formatt!(4; "if(__b &= ((__word & __mask) == ({})); !__b) return 0;", stringify!($assert_val));
        __s += &formatt!(3; "}}\n"); 
        __s += &parse_dyn_check_inside!(@$($rest)* );
        __s
    }};

    (@ $bit:tt => $assert_val:expr ; $($rest:tt)*) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(4; "uint32_t __word = (uint32_t)(data[num_items] >> ({}));", stringify!($bit));
        __s += &formatt!(4; "if(__b &= ((__word & 1) == ({})); !__b) return 0;", stringify!($assert_val));
        __s += &formatt!(3; "}}\n"); 
        __s += &parse_dyn_check_inside!(@$($rest)* );
        __s
    }};

    (@ ENCODE( $($_tt:tt)* ) ; $($rest:tt)* ) => {
        parse_dyn_check_inside!(@$($rest)* )
    };

    (@) => {{
        String::new()
    }};
    () => {{
        String::new()
    }};
}
#[macro_export]
macro_rules! parse_dyn_init_inside {
    // This will look for the encode statement inside
    (@ $left_bound:tt .. $right_bound:expr => $assert_val:expr ; $($rest:tt)*) => {
        parse_dyn_init_inside!(@$($rest)* )
    };

    (@ $bit:tt => $assert_val:expr ; $($rest:tt)*) => {
        parse_dyn_init_inside!(@$($rest)* )
    };
        
    (@ ENCODE($left_bound:tt .. $right_bound:expr => $encode_id:ident) ; $($rest:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "for(int _i = 0; _i < MAX_DYN; ++_i)\n");
        __s += &formatt!(3; "{}.assign(&data[_i], {}, {});\n", stringify!($encode_id), stringify!($left_bound), stringify!($right_bound));

        __s += &parse_dyn_init_inside!(@$($rest)* );
        __s
    }};

    (@) => {{
        String::new()
    }};
    () => {{
        String::new()
    }};
}

