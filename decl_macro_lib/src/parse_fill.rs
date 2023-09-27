#[macro_export] 
macro_rules! munch_fill { 
    (@for ( $loop_left:tt <= $loop_index:ident < $loop_right:expr ) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left .. $loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &munch_fill!(@ [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &munch_fill!(@ $($other_fields)*);
        __s
    }};

    // If encountering either a primitive or composed - just call their __fill()
    (@ $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{}", stringify!($field_name));
        $( __s += &format!("_{}", $loop_index); )?
        __s += &format!(".__fill(__event_handle, bytes_available, bytes_read_sub);\n");
        __s += &formatt!(2;"__event_handle += bytes_read_sub;\n");
        __s += &formatt!(2;"bytes_read += bytes_read_sub;\n\n");
        __s += &munch_fill!(@ $($other_fields)*);
        __s
    }}; 
    // Following three rules expand to the rule above:
    (@ $([[ $loop_index:expr ]])? $field_type:ident ( $($field_generic:expr),* ) $field_name:ident ; $($other_fields:tt)* ) => {
        munch_fill!(@ $([[$loop_index]])? $field_type $field_name ; $($other_fields)* )
    };
    (@ $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        munch_fill!(@ $([[$loop_index]])? $field_type $field_name ; $($other_fields)* )
    };
    (@ $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_fill!(@ $([[$loop_index]])? $field_type $field_name ; $($other_fields)* )
    };
    
    // dyn! fields keep filling the array until either array is full, buffer is over or condition
    // is violated.
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $(($($field_generic:expr),*))? $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
    }};
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        munch_condition!( $name $($other_fields)*)
    };
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_condition!( $name $($other_fields)*)
    };

    (@ $([[ $loop_index:expr ]])? ) => {{
        String::new()
    }};

}
