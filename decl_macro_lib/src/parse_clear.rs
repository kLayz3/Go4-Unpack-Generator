#[macro_export]
macro_rules! munch_clear { 
    (@ for ( $loop_left:tt <= $loop_index:ident < $loop_right:expr ) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left .. $loop_right {
            __s += &munch_clear!(@ [[i]] $($loop_body)*);
        }
        __s += &munch_clear!(@ $($other_fields)*);
        __s
    }};

    (@dyn! $([max = $max_dyn:expr])? $field_type:ident $(($($field_generic:expr),*))? $field_name:ident ; $($other_fields:tt)* ) => {
        let mut __s = String::new();
        __s += &formatt!(2; "for(int __ii = 0; __ii < N_{}; ++__ii)\n", stringify!($field_name));
        __s += &formatt!(3; "{}[__ii].__clear();\n", stringify!($field_name));
        __s += &formatt!(2; "}}\n");
        __s += &formatt!(2; "N_{} = 0;\n", stringify!($field_name));
        __s += &munch_clear!(@ $($other_fields)*);
        __s
    };
    // Next two rules expand to the rule above.
    (@dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        munch_clear!(@dyn! $([max = $max_dyn])? $field_type:ident $field_name:ident ; $($other_fields:tt)* ) 
    };
    (@dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_clear!(@dyn! $([max = $max_dyn])? $field_type:ident $field_name:ident ; $($other_fields:tt)* ) 
    };

    // All the fields just call their own __clear()
    (@$([[ $loop_index:expr ]])? $field_type:ident $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{}", stringify!($field_name));
        $( __s += &format!("_{}", $loop_index); )?
        __s += ".__clear();\n";
        __s += &munch_clear!(@ $([[$loop_index]])? $($other_fields)*);
        __s
    }};
    // Next two rules expand to the rule above.
    (@$([[ $loop_index:expr ]])? $field_type:ident ( $($field_generic:expr)? ) $field_name:ident ; $($other_fields:tt)* ) => { 
        munch_clear!(@ $([[$loop_index]])? $field_type $field_name ; $($other_fields)*);
    };

    (@$([[ $loop_index:expr ]])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => { 
        munch_clear!(@ $([[$loop_index]])? $field_type $field_name ; $($other_fields)*);
    };
    // Return back at the max depth
    (@$([[ $loop_index:expr ]])? ) => {{
        String::new()
    }}
}
