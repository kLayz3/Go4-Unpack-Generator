#[macro_export]
macro_rules! munch_size {
    (@ for ( $loop_left:tt <= $loop_index:ident < $loop_right:expr ) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left .. $loop_right {
            __s += &munch_size!(@$($loop_body)*);
        }
        __s += &munch_size!(@$($other_fields)*);
        __s
    }};
    // Dyn objects have minimal size 0, skip them
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        munch_size!(@$($other_fields:tt)* ) 
    };
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_size!(@$($other_fields:tt)* ) 
    };
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $(($($field_generic:expr),*))? $field_name:ident ; $($other_fields:tt)* ) => {
        munch_size!(@$($other_fields:tt)* ) 
    };

    (@ $field_type:ident $field_name:ident $( { $($condition_body:tt)* } )? ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "struct_size += __{}<>::__min_size();\n", stringify!($field_type));
        __s += &munch_size!(@$($other_fields)* );
        __s
    }};

    (@ $field_type:ident ( $($field_generic:expr),* ) $field_name:ident ; $($other_fields:tt)* ) => {{ 
        let mut __s = String::new();
        __s += &formatt!(2; "struct_size += __{}<", stringify!($field_type));
        let mut generics_added = 0;
        $(
        __s += &format!("{},", stringify!($field_generic)); 
        generics_added += 1;
        )*
        if generics_added > 0 { // remove last comma
            __s = __s[0.. __s.len()-1].to_string();
        }

        __s += &format!(">::__min_size();\n");
        __s += &munch_size!(@$($other_fields)* );
        __s
    }};

    (@) => {{
        String::new()
    }};
}
