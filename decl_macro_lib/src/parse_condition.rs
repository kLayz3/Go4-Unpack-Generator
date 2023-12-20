#[macro_export] 
macro_rules! parse_condition {
    // `$name` is the structure identifier passed to this macro by the `parse_struct!` invocation.
    
    // Skip parsing MEMBER decl:
    ($name:ident MEMBER( $($x:tt)* ); $($other_fields:tt)* ) => {
        parse_condition!( $name $($other_fields)*)
    };
    // Dynamic objects don't participate in check_event(), their data is checked during filling.
    ( $name:ident dyn! $([max = $max_dyn:expr])? $field_name:ident = $field_type:ident ($($field_generic:expr),*)  ; $($other_fields:tt)* ) => {
        parse_condition!( $name $($other_fields)*)
    };
    ( $name:ident dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        parse_condition!( $name $($other_fields)*)
    };
    ( $name:ident dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        parse_condition!( $name $($other_fields)*)
    };

    ( $name:ident $([[$loop_index:expr]])? ) => {{
        String::new()
    }};

     // If encountering a for loop, repeat the body:
    ( $name:ident for($loop_left:tt <= $loop_index:ident < $loop_right:expr) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left..$loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &parse_condition!( $name [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &parse_condition!( $name $($other_fields)*);
        __s
    }};

    // Process `MATCH` tokens. Possibly inside a `for` loop 
    ( $name:ident $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident = MATCH($assert_val:expr) ; $($other_fields:tt)*  ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "if(__b &= ({}", stringify!($field_name));
        $( // Add loop_index as underscore
        __s += &format!("_{}", $loop_index);
        )?
        __s += &format!(" == ({})); __!b) {{\n", stringify!($assert_val));
        __s += &formatt!(4; "printerr(\"Event mismatch! Trying to `MATCH` in structure: {}::{}", stringify!($name), stringify!($field_name));
        $(
        __s += &format!("_{} (inside `for`)", $loop_index);
        )?
        __s += &format!(".\");\n");
        __s += &formatt!(4; "return 0;\n");
        __s += &formatt!(3; "}}\n"); 
        __s += &formatt!(2; "}}\n"); 
        
        __s += &parse_condition!( $name $([[$loop_index]])? $($other_fields)*);
        __s
    }};
       
    // Fields without condition block are skipped.
    ( $name:ident $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident ; $($other_fields:tt)* ) => {
        parse_condition!( $name $([[$loop_index]])? $($other_fields)*)
    };

    // Non-primitive fields just call their own `check_event()`
    ( $name:ident $([[$loop_index:expr]])? 
     $field_name:ident = $field_type:ident ( $($field_generic:expr),* ) ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "if(__b &= this->{}", stringify!($field_name));
        $(__s += &format!("_{}", $loop_index); )?
        __s += ".check_event(); !__b) return __b;\n";
        __s += &parse_condition!( $name $([[$loop_index]])? $($other_fields)*);
        __s
    }};
   
    // Once the condition body is identified, go inside.
    // Bundle the $name with $field name for a unique tag.
    ( $name:ident $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident { $($inside:tt)* } ; $($other_fields:tt)*) => {{
        let mut __s = String::new();
        __s += &parse_condition_inside!( ($name,$field_name) $([[$loop_index]])? => $($inside)* );
        __s += &parse_condition!( $name $([[$loop_index]])? $($other_fields)* );
        __s
    }};
}
#[macro_export]
macro_rules! parse_condition_inside {
    // A field could encounter 4 different possible rules:
    // U32 NAMED {
    //           0..15 => 0xfefe; // Ranged assert    (1)
    //              19 => 0x0;    // Bit assert       (2)
    //   ENCODE(21..31 => id);    // ENCODE directive (3)
    //   assert($expr);           // assert directive (4)
    // };
    // On either the left or the right side of => can also be the generic parameter (of main structure), 
    // a loop value so just paste the token and 'trust' the user it's implied somewhere.
    // A custom compile_error should be raised at this point then!
    // `assert` is added to matcher to enable various bound checking in the unpacked values, and 
    // flag the structure as unpacked with a custom error message

    // Possibility (4):
    ( ($name:ident, $field_name:ident) $([[$loop_index:expr]])? =>
     assert!( $ee:expr ) ; $($rest:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "if({}) {{\n", stringify!($ee));
        __s += &formatt!(4; "printerr(\"Assert `{}` failed in structure %s. {}\", this->__self_name);\n", stringify!($ee));
        __s ++ &formatt!(4; "return false;\n");
        __s += &formatt!(2; "}}\n");
        __s += &parse_condition_inside!( ($name,$field_name) $([[$loop_index]])? => $($rest)* );
        __s
    }};
    // Possibility (3): skip
    ( ($name:ident, $field_name:ident) $([[$loop_index:expr]])? =>
     ENCODE( $($_tt:tt)* ) ; $($rest:tt)* ) => {
        parse_condition_inside!( ($name,$field_name) $([[$loop_index]])? => $($rest)* )
    };

    // Possibility (1)
    ( ($name:ident, $field_name:ident) $([[$loop_index:expr]])? =>
        $left_bound:tt .. $right_bound:expr => $assert_val:expr ; $($rest:tt)*) 
      => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "uint32_t __mask = (uint32_t)((1ull << ({} - ({}))) - 1);\n", stringify!($right_bound), stringify!($left_bound));
        __s += &formatt!(3; "uint32_t __word = (uint32_t)(this->{}", stringify!($field_name));
        $( __s += &format!("_{}", $loop_index); )?
        __s += &format!(" >> ({}));\n" , stringify!($left_bound));
        __s += &formatt!(3; "if(__b &= ((__word & __mask) == ({})); !__b) {{\n", stringify!($assert_val));
        __s += &formatt!(4; "printerr(\"Event mismatch! In structure: {}.{}", stringify!($name), stringify!($field_name));
        $( __s += &format!("_{} (inside `for`).", $loop_index); )?
        __s += &format!(".\");\n");
        __s += &formatt!(4; "printerr(\"Expected 0x%8x, found: 0x%8x.\\n\", {}, __word & __mask);\n", stringify!($assert_val));
        __s += &formatt!(4; "return 0;\n");
        __s += &formatt!(3; "}}\n"); 
        __s += &formatt!(2; "}}\n"); 
        
        __s += &parse_condition_inside!( ($name,$field_name) $([[$loop_index]])? => $($rest)*);
        __s 
    }};

    // Possibility (2)
    ( ($name:ident,$field_name:ident) $([[$loop_index:expr]])? =>
        $bit:tt => $assert_val:expr ; $($rest:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "uint32_t __word = (uint32_t)(this->{}", stringify!($field_name));
        $( __s ++ &format!("_{}", $loop_index); )?
        __s += &format!(" >> ({}));\n" , stringify!($bit));
        __s += &formatt!(3; "if(__b &= ((__word & 1) == ({})); !__b) {{\n", stringify!($assert_val));
        __s += &formatt!(4; "printerr(\"Event mismatch! In structure: {}.{}", stringify!($name), stringify!($field_name));
        $( __s += &format!("_{} (inside `for`).", $loop_index) )?
        __s += &format!(".\");\n");
        __s += &formatt!(4; "printerr(\"Expected 0x%x, found: 0x%x.\\n\", {}, __word & 1);\n", stringify!($assert_val));
        __s += &formatt!(4; "return 0;\n");
        __s += &formatt!(3; "}}\n"); 
        __s += &formatt!(2; "}}\n"); 
        
        __s += &parse_condition_inside!( ($name,$field_name) $([[$loop_index]])? => $($rest)*);
        __s 
    }};

    ( ($name:ident,$field_name:ident) $([[$loop_index:expr]])? => ) => {{
        String::new()
    }};

    () => {{
        String::new()
    }}

}
