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
    // Dynamic fields just call their own clear
    (@dyn! $([max = $max_dyn:expr])? $field_name:ident = $field_type:ident ($($field_generic:expr),*) ; $($other_fields:tt)* ) => {
        munch_clear!(@ $field_type $field_name ; $($other_fields)*)
    }; 
    (@dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        munch_clear!(@ $field_type $field_name ; $($other_fields)*)
    };
    (@dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_clear!(@ $field_type $field_name ; $($other_fields)*)
    };

    // All the fields just call their own `clear()`
    (@$([[ $loop_index:expr ]])? $field_type:ident $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{}", stringify!($field_name));
        $( __s += &format!("_{}", $loop_index); )?
        __s += ".clear();\n";
        __s += &munch_clear!(@ $([[$loop_index]])? $($other_fields)*);
        __s
    }};
    // Next three rules expand to the rule above.
    (@$([[ $loop_index:expr ]])? $field_name:ident = $field_type:ident ( $($field_generic:expr)? ) ; $($other_fields:tt)* ) => { 
        munch_clear!(@ $([[$loop_index]])? $field_type $field_name ; $($other_fields)*);
    };

    (@$([[ $loop_index:expr ]])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => { 
        munch_clear!(@ $([[$loop_index]])? $field_type $field_name ; $($other_fields)*);
    };

    (@$([[ $loop_index:expr ]])? $field_type:ident $field_name:ident = MATCH($assert_val:expr) ; $($other_fields:tt)*  ) => {
        munch_clear!(@ $([[$loop_index]])? $field_type $field_name ; $($other_fields)*);
    };
    // Return back at the max depth
    (@$([[ $loop_index:expr ]])? ) => {{
        String::new()
    }}
}
