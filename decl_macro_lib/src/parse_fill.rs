#[macro_export] 
macro_rules! parse_fill { 
    // Create the body of the `fill(..)` method 

    // Skip MEMBER declarations
    (@MEMBER( $($x:tt)* ); $($other_fields:tt)* ) => { 
        parse_fill!(@$($other_fields)*)
    };
    // Skip placeholders
    (@local! $field_type:ident $field_name:ident ; $($other_fields:tt)* ) => {
        parse_fill!(@$($other_fields)*)
    };
    (@for ( $loop_left:tt <= $loop_index:ident < $loop_right:expr ) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left .. $loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &parse_fill!(@ [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &parse_fill!(@ $($other_fields)*);
        __s
    }};

    // If encountering either a primitive or composed - just call their fill()
    (@ $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{}", stringify!($field_name));
        $( __s += &format!("_{}", $loop_index); )?
        __s += &format!(".fill(event_handle, bytes_available, bytes_read_sub);\n");
        __s += &formatt!(2;"event_handle += bytes_read_sub;\n");
        __s += &formatt!(2;"bytes_read += bytes_read_sub;\n");
        __s += &parse_fill!(@ $($other_fields)*);
        __s
    }}; 
    // Following three rules expand to the rule above:
    (@ $([[ $loop_index:expr ]])? $field_name:ident = $field_type:ident ( $($field_generic:tt)* ) ; $($other_fields:tt)* ) => {
        parse_fill!(@ $([[$loop_index]])? $field_type $field_name ; $($other_fields)* )
    };
    (@ $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        parse_fill!(@ $([[$loop_index]])? $field_type $field_name ; $($other_fields)* )
    };
    (@ $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        parse_fill!(@ $([[$loop_index]])? $field_type $field_name ; $($other_fields)* )
    };
    // dyn! fields also just call their own `fill`
    (@ dyn! $([max = $max_dyn:expr])? $field_name:ident = $field_type:ident ( $($field_generic:tt)* ) ; $($other_fields:tt)* ) => {
        parse_fill!(@ $field_type $field_name ; $($other_fields)*)
    };
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        parse_fill!(@ $field_type $field_name ; $($other_fields)*)
    };
    (@ dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        parse_fill!(@ $field_type $field_name ; $($other_fields)*)
    };

    (@ $([[ $loop_index:expr ]])? ) => {{
        String::new()
    }};

}
