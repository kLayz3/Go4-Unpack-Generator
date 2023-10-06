#[allow(dead_code)] pub const MAX_DYN_DEFAULT: usize = 128;

#[allow(unused_mut)]
#[macro_export]
macro_rules! parse_fields {
    // Parse `MEMBER` declarations, $num_of_items can be a template parameter!
    (@MEMBER($type:ident $name:ident $([$num_of_items:expr])* ); $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(1; "Go4UnpackPtr<{}> {}", stringify!($type), stringify!($name));
         $( __s += &format!("[{}]", stringify!($num_of_items)); )*
        __s += ";\n";
        __s += &parse_fields!(@ $($other_fields)*);
        __s
    }};
    
    // Expand fields defined in a `for`  
    (@for ( $loop_left:tt <= $loop_index:ident < $loop_right:expr ) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left .. $loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &parse_fields!(@ [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &parse_fields!(@ $($other_fields)*);
        __s
    }};
    
    // Dynamic fields with capacity $max_dyn hold an array. Cannot be in a `for`. 
    // Possible for structure fields without `{}` block or primitives with mandatory `{}` block or
    // `MATCH` assignment 
    // max_dyn can be a template parameter!! 
    (@dyn! $([max = $max_dyn:expr])? $field_name:ident = $field_type:ident ( $($fgen_name:ident = $fgen_val:expr),* ) ; $($other_fields:tt)* ) => {{
        let mut max_dyn = stringify!(MAX_DYN_DEFAULT);
        $( max_dyn = stringify!($max_dyn); )?
        let mut __s = String::new();
        __s += &format!("#define _MAX_DYN {}\n", max_dyn);
        __s += &parse_dyn_decl!($field_type ( $($fgen_val),* ) $field_name);
        __s += &parse_dyn_init!();
        __s += &parse_dyn_common!($field_type $field_name);
        __s += &parse_dyn_check!($field_type $field_name);
        __s += &formatt!(1; "}} {};\n", stringify!($field_name));
        __s += &format!("#undef _MAX_DYN\n");
        __s += &parse_fields!(@ $( $other_fields )*);
        __s
    }}; 
    // Next two rules expand similarly, for a field tagged with `{}` or a `MATCH`
    (@dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {{
        let mut max_dyn = MAX_DYN_DEFAULT;
        $( max_dyn = $max_dyn as usize; )?
        let mut __s = String::new();
        __s += &format!("#define _MAX_DYN {}\n", max_dyn);
        __s += &parse_dyn_decl!($field_type () $field_name);
        __s += &parse_dyn_init!($field_type $field_name { $($condition_body)* });
        __s += &parse_dyn_common!($field_type $field_name);
        __s += &parse_dyn_check!($field_type $field_name { $($condition_body)* });
        __s += &formatt!(1; "}} {};\n", stringify!($field_name));
        __s += &format!("#undef _MAX_DYN\n");
        __s += &parse_fields!(@ $( $other_fields )*);
        __s
    }};
    (@dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($match_val:expr) ; $($other_fields:tt)* ) => {{
        let mut max_dyn = MAX_DYN_DEFAULT;
        $( max_dyn = $max_dyn as usize; )?
        let mut __s = String::new();
        __s += &format!("#define _MAX_DYN {}\n", max_dyn);
        __s += &parse_dyn_decl!($field_type () $field_name);
        __s += &parse_dyn_init!();
        __s += &parse_dyn_common!($field_type $field_name);
        __s += &parse_dyn_check!($field_type $field_name = MATCH($match_val));
        __s += &formatt!(1; "}} {};\n", stringify!($field_name));
        __s += &format!("#undef _MAX_DYN\n");
        __s += &parse_fields!(@ $( $other_fields )*);
        __s
    }};

    // Fields with `MATCH` cannot be a generic or hold a {} block.
    (@$([[ $loop_index:expr ]])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        parse_fields!(@ $([[$loop_index]])? $field_type $field_name ; $($other_fields)* )
    };

    // Non-primitive fields are of the format: name = type(gen_name = gen_val, ...);
    (@$([[ $loop_index:expr ]])? $field_name:ident = $field_type:ident ( $($fgen_name:ident = $fgen_val:expr),* ) ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(1; "_{}<", stringify!($field_type));
        $( __s += &format!("{},", stringify!($fgen_val)); )*
        __s += &format!("void> {}", stringify!($field_name));
        $( // Add loop index to the name, if it is supplied
        __s += &format!("_{}", $loop_index);
        )?
        __s += ";\n";
        __s += &parse_fields!(@ $([[ $loop_index ]])? $( $other_fields )*);
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
        __s += &parse_fields!(@ $([[ $loop_index ]])? $( $other_fields )*);
        __s   
    }};

    // Return at the max depth of recursion
    (@$([[ $loop_index:expr ]])? ) => {{
        String::new()
    }};

    () => {{
        String::new()
    }};
}

include!("parse_dyn.rs");
