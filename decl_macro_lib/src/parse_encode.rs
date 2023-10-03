#[macro_export]
macro_rules! munch_encode {
    // This macro will search for all ENCODE statements and create the init() method
    // which should be called only once at runtime.
    // It creates appropriate pointers to encoded data words

     // If encountering a for loop, repeat the body:
    (@ for($loop_left:tt <= $loop_index:ident < $loop_right:expr) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left..$loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &munch_encode!(@ [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &munch_encode!(@ $($other_fields)*);
        __s
    }};
    // Primitive fields with no condition block are skipped, they can't encode anything 
    (@ $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident ; $($other_fields:tt)* ) => {
        munch_encode!(@$([[$loop_index]])? $($other_fields)*)
    };
    // Non-primitives just call their own `init()`.
    (@ $([[$loop_index:expr]])? 
     $field_name:ident = $field_type:ident ( $($field_generic:expr),* ) ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "__{}", stringify!($field_name));
        $( __s += &format!("_{}", $loop_index); )?
        __s += &format!(".init();\n");
        __s += &munch_encode!(@ $([[$loop_index]])? $($other_fields)*);
        __s
    }};

    // MATCH'ed primitive fields also encode anything.
    (@ $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident = MATCH($match_val:expr); $($other_fields:tt)* ) => {
        munch_encode!(@$([[$loop_index]])? $field_type $field_name ; $($other_fields)*)
    };
    
    // Go inside the {} body where ENCODE's could live. Pass `$field_name => ` as a tag
    (@ $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident { $($inside:tt)* } ; $($other_fields:tt)*) => {{
        let mut __s = String::new();
        __s += &munch_encode_inside!( $field_name $([[$loop_index]])? => $($inside)* );
        __s += &munch_encode!(@ $([[$loop_index]])? $($other_fields)* );
        __s
    }};

    // dyn objects without {} block don't encode anything new.
    (@ dyn! $([max = $max_dyn:expr])? $field_name:ident = $field_type:ident ($($field_generic:expr),*) ; $($other_fields:tt)* ) => {
        munch_encode!(@ $($other_fields)* )
    };
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_encode!(@ $($other_fields)* )
    };
    // Once found, go inside the {} block for `dyn!` and parse all encodes.
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($inside:tt)* } ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        let mut max_dyn: usize = MAX_DYN_DEFAULT;
        $( max_dyn = $max_dyn as usize; )?
        for(i = 0;)
        __s += &munch_encode_inside_dyn!( $field_name $(, $max_dyn)? => $($inside)* );
        __s += &munch_encode!(@ $($other_fields)*);
        __s
    }};

    (@ $([[$loop_index:expr]])? ) => {{
        String::new()
    }};
}

#[macro_export]
macro_rules! munch_encode_inside {
    ( $field_name:ident $([[$loop_index:expr]])? =>
     ENCODE($left_bound:tt .. $right_bound:expr => $encode_id:ident $( [$encode_index:expr] )? ) ; $($rest:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3;"void* _p = (void*)&{}", stringify!($field_name));
        $(__s += &format!("_{}", $loop_index); )?
        __s += ";\n";
        __s += &formatt!(3; "Go4UnpackPtr _ptr({}, {}, _p);\n", stringify!($left_bound), stringify!($right_bound));
        __s += &formatt!(3;"m.emplace(std::make_pair(\"{}", stringify!($encode_id));
        $(__s += &format!("[{}]",stringify!($encode_index)); )?
        __s += "\", _ptr));\n";
        __s += &formatt!(2; "}}\n");
        __s += &munch_encode_inside!( $field_name $([[$loop_index]])? => $($rest)*);
        __s
    }};
    // Ignore condition statement (1)
    ( $field_name:ident $([[$loop_index:expr]])? =>
        $left_bound:tt .. $right_bound:expr => $assert_val:expr ; $($rest:tt)*) => {
        munch_encode_inside!( $field_name $([[$loop_index]])? => $($rest)* )
    }; 
    // Ignore condition statement (2)
    ( $field_name:ident $([[$loop_index:expr]])? =>
        $bit:tt => $assert_val:expr ; $($rest:tt)* ) => { 
        munch_encode_inside!( $field_name $([[$loop_index]])? => $($rest)*)
    };
    ( $field_name:ident $([[$loop_index:expr]])? => ) => {{
        String::new()
    }};
}

#[macro_export]
macro_rules! munch_encode_inside_dyn {
    // Dynamic Fields:
    ( $field_name:ident $(, $max_dyn:expr )? =>
     ENCODE($left_bound:tt .. $right_bound:expr => $encode_id:ident ) ; $($rest:tt)* ) => {{
        let mut __s = String::new();
        let mut max_dyn: usize = MAX_DYN_DEFAULT;
        $( max_dyn = $max_dyn as usize; )?
         __s += &formatt!(2; "for(int i = 0; i < {}; ++i) {{\n", max_dyn);
        __s += &formatt!(3;"void* _p = (void*)&{}[i];\n", stringify!($field_name));
        __s += &formatt!(3; "Go4UnpackPtr _ptr({}, {}, _p);\n", stringify!($left_bound), stringify!($right_bound));
        __s += &formatt!(3;"m.emplace(std::make_pair(\"{}[i]\", _ptr));\n", stringify!($encode_id));
        __s += &formatt!(2; "}}\n");
        __s += &munch_encode_inside!( $field_name $(, $max_dyn )? => $($rest)*);
        __s
    }};
    ( $field_name:ident $(, $max_dyn:expr )? =>
        $left_bound:tt .. $right_bound:expr => $assert_val:expr ; $($rest:tt)*) => {
        munch_encode_inside!( $field_name $(, $max_dyn )? => $($rest)*)
    }; 
    ( $field_name:ident $(, $max_dyn:expr )? =>
        $bit:expr => $assert_val:expr ; $($rest:tt)* ) => { 
        munch_encode_inside!( $field_name $(, $max_dyn )? => $($rest)*)
    };
    ( $field_name:ident $(, $max_dyn:expr )? => ) => {{
        String::new()
    }};
}
