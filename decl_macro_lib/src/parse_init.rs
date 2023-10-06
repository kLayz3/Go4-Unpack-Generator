#[macro_export]
macro_rules! parse_init {
    // This macro will search for all ENCODE statements and create the `init()` method
    // which should be called only once at initialization.
    // It binds MEMBER pointers to selected data words
    
    // Skip parsing MEMBER decl:
    (@MEMBER( $($x:tt)* ); $($other_fields:tt)* ) => {
        parse_init!(@$($other_fields)*)
    };

     // If encountering a `for` statement, repeat the body:
    (@ for($loop_left:tt <= $loop_index:ident < $loop_right:expr) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left..$loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &parse_init!(@ [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &parse_init!(@ $($other_fields)*);
        __s
    }};
    // Primitive fields with no condition block are skipped, they can't encode anything 
    (@ $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident ; $($other_fields:tt)* ) => {
        parse_init!(@$([[$loop_index]])? $($other_fields)*)
    };
    // MATCH'ed primitive fields also can't encode anything.
    (@ $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident = MATCH($match_val:expr); $($other_fields:tt)* ) => {
        parse_init!(@$([[$loop_index]])? $field_type $field_name ; $($other_fields)*)
    };

    // Non-primitives just call their own `init()`.
    (@ $([[$loop_index:expr]])? 
     $field_name:ident = $field_type:ident ( $($field_generic:tt)* ) ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "__{}", stringify!($field_name));
        $( __s += &format!("_{}", $loop_index); )?
        __s += &format!(".init();\n");
        __s += &parse_init!(@ $([[$loop_index]])? $($other_fields)*);
        __s
    }};
        
    // Go inside the {} body where ENCODE's could live. Pass `$field_name => ` as a tag
    (@ $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident { $($inside:tt)* } ; $($other_fields:tt)*) => {{
        let mut __s = String::new();
        __s += &parse_init_inside!($([[$loop_index]])? $field_name => $($inside)* );
        __s += &parse_init!(@ $([[$loop_index]])? $($other_fields)* );
        __s
    }};

    // dyn objects just call their own `init()`
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($inside:tt)* } ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{}.init();\n", stringify!($field_name));
        __s += &parse_init!(@ $($other_fields)*);
        __s
    }};
    // Next two rules expand to the rule above
    (@ dyn! $([max = $max_dyn:expr])? $field_name:ident = $field_type:ident ( $($field_generic:tt)* ) ; $($other_fields:tt)* ) => {
        parse_init!(@dyn! $([max = $max_dyn])? $field_type $field_name { } ; $($other_fields)* )
    };
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        parse_init!(@dyn! $([max = $max_dyn])? $field_type $field_name { } ; $($other_fields)* )
    };
    
    (@ $([[$loop_index:expr]])? ) => {{
        String::new()
    }};
}

#[macro_export]
macro_rules! parse_init_inside {
    ($([[$loop_index:expr]])? $field_name:ident =>
     ENCODE($left_bound:tt .. $right_bound:expr => $encode_id:ident $( [$encode_index:expr] )? ) ; $($rest:tt)* ) => {{

        let mut __s = String::new();
        let mut _member_name = stringify!($encode_id);
        $( _member_name += &format!("[{}]", stringify!($encode_index)); )*

        let mut _assign_args = format!("&{}", stringify!($field_name));
        $( _assign_args += &format!("_{}", $loop_index); )*
        _assign_args += &format!(", {}, {}", stringify!($left_bound), stringify!($right_bound));

        __s += &formatt!(2; "{}.assign({});\n", _member_name, _assign_args); 
        __s += &parse_init_inside!( $field_name $([[$loop_index]])? => $($rest)*);
        __s
    }};
    // Ignore condition statement (1)
    ($([[$loop_index:expr]])? $field_name:ident =>
        $left_bound:tt .. $right_bound:expr => $assert_val:expr ; $($rest:tt)*) => {
        parse_init_inside!( $field_name $([[$loop_index]])? => $($rest)* )
    }; 
    // Ignore condition statement (2)
    //
    ($([[$loop_index:expr]])? $field_name:ident =>
        $bit:tt => $assert_val:expr ; $($rest:tt)* ) => { 
        parse_init_inside!( $field_name $([[$loop_index]])? => $($rest)*)
    };
    ($([[$loop_index:expr]])? $field_name:ident => ) => {{
        String::new()
    }};
}

#[macro_export]
macro_rules! parse_init_inside_dyn {
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
        __s += &parse_init_inside!( $field_name $(, $max_dyn )? => $($rest)*);
        __s
    }};
    ( $field_name:ident $(, $max_dyn:expr )? =>
        $left_bound:tt .. $right_bound:expr => $assert_val:expr ; $($rest:tt)*) => {
        parse_init_inside!( $field_name $(, $max_dyn )? => $($rest)*)
    }; 
    ( $field_name:ident $(, $max_dyn:expr )? =>
        $bit:expr => $assert_val:expr ; $($rest:tt)* ) => { 
        parse_init_inside!( $field_name $(, $max_dyn )? => $($rest)*)
    };
    ( $field_name:ident $(, $max_dyn:expr )? => ) => {{
        String::new()
    }};
}
