#[allow(dead_code)] const MAX_DYN_DEFAULT: usize = 100;

#[macro_export]
macro_rules! munch_fields {
    // Expand fields defined in a `for`  
    (@for ( $loop_left:tt <= $loop_index:ident < $loop_right:expr ) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left .. $loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &munch_fields!(@ [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &munch_fields!(@ $($other_fields)*);
        __s
    }};
    
    // Dynamic fields with capacity $max_dyn hold an array. Cannot be in a `for`. 
    // Possible for structure fields without {} block or primitives with {} block
    (@dyn! $([max = $max_dyn:expr])? $field_type:ident $(($($field_generic:expr),*))? $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(1; "int N_{};\n", stringify!($field_name));
        let mut generics_added = 0;
        __s += &formatt!(1; "__{}<", stringify!($field_type));
        $(
        __s += &format!("{},", stringify!($field_generic)); 
        generics_added += 1;
        )*
        if generics_added > 0 { // remove last comma
            __s = __s[0.. __s.len()-1].to_string();
        }
        __s += &format!("> {}", stringify!($field_name));
        let mut max_dyn: usize = MAX_DYN_DEFAULT;
        $( max_dyn = $max_dyn as usize; )?
        __s += &format!("[{}];\n", max_dyn);
        __s += &munch_fields!(@ $([[ $loop_index ]])? $( $other_fields )*);
        __s
    }};
    // Next two rules expand to the rule above.
    (@dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        munch_fields!(@ dyn! $([max = $max_dyn])? $field_type:ident $field_name:ident ; $($other_fields:tt)* ) 
    };
    (@dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_fields!(@ dyn! $([max = $max_dyn])? $field_type:ident $field_name:ident ; $($other_fields:tt)* ) 
    };

    // Fields with `MATCH` cannot be a generic or hold a {} block.
    (@$([[ $loop_index:expr ]])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_fields!($([[$loop_index]])? $field_type $field_name ; $($other_fields)* )
    };

    // Generic'ed fields cannot hold condition block.
    (@$([[ $loop_index:expr ]])? $field_type:ident ( $($field_generic:expr),* ) $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        let mut generics_added = 0;
        __s += &formatt!(1; "__{}<", stringify!($field_type));
        $(
        __s += &format!("{},", stringify!($field_generic)); 
        generics_added += 1;
        )*
        if generics_added > 0 { // remove last comma
            __s = __s[0.. __s.len()-1].to_string();
        }
        __s += &format!("> {}", stringify!($field_name));
        $( // Add loop index to the name, if it is supplied
        __s += &format!("_{}", $loop_index);
        )?
        __s += ";\n";
        __s += &munch_fields!(@ $([[ $loop_index ]])? $( $other_fields )*);
        __s
    }};

    // Condition block can only be held by a primitive
    (@$([[ $loop_index:expr ]])? $field_type:ident $field_name:ident $( { $($condition_body:tt)* } )? ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(1; "__{}<> {}", stringify!($field_type), stringify!($field_name));
        $( // Add loop index to the name, if it is supplied
        __s += &format!("_{}", $loop_index);
        )?
        __s += ";\n";
        __s += &munch_fields!(@ $([[ $loop_index ]])? $( $other_fields )*);
        __s   
    }};

    // At the max depth of recursion, return and pop the stack frames
    (@$([[ $loop_index:expr ]])? ) => {{
        String::new()
    }};
    () => {{
        String::new()
    }};
}
