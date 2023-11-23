#[macro_export]
macro_rules! parse_size {
    // Skip parsing MEMBER decl:
    (@MEMBER( $($x:tt)* ); $($other_fields:tt)* ) => {
        parse_size!(@$($other_fields)*)
    };
    // Skip placeholders
    (@local! $field_type:ident $field_name:ident ; $($other_fields:tt)* ) => {
        parse_size!(@$($other_fields)*)
    };

    (@ for ( $loop_left:tt <= $loop_index:ident < $loop_right:expr ) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left .. $loop_right {
            __s += &parse_size!(@$($loop_body)*);
        }
        __s += &parse_size!(@$($other_fields)*);
        __s
    }};

    // Dynamic objects have minimal size 0; aren't included
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        parse_size!(@$($other_fields)* ) 
    };
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        parse_size!(@$($other_fields)* ) 
    };
    (@ dyn! $([max = $max_dyn:expr])? $field_name:ident = $field_type:ident ( $($field_generic:tt)* ) ; $($other_fields:tt)* ) => {
        parse_size!(@$($other_fields)* ) 
    };
     
    (@ $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        parse_size!(@ $field_type $field_name ; $($other_fields)*)
    };
    (@ $field_type:ident $field_name:ident $( { $($condition_body:tt)* } )? ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "struct_size += __{}<>::min_size();\n", stringify!($field_type));
        __s += &parse_size!(@$($other_fields)* );
        __s
    }};

    (@ $field_name:ident =  $field_type:ident ( $($fgen_name:ident = $fgen_val:expr),* ) ; $($other_fields:tt)* ) => {{ 
        let mut __s = String::new();
        __s += &formatt!(2; "struct_size += __{}<", stringify!($field_type));
        $( __s += &format!("{},", stringify!($fgen_val)); )*
        __s += &format!("void>::min_size();\n");
        __s += &parse_size!(@$($other_fields)* );
        __s
    }};

    (@) => {{
        String::new()
    }};
}
