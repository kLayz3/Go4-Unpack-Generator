#[macro_export] 
macro_rules! munch_condition {
    // `$name` is the structure identifier passed to this macro by the main invocation.
    // Tokens belonging to MEMBER annotations are ignored
    ( $name:ident MEMBER( $($__tt:tt)* ) $($rest:tt)*) => {
        munch_condition!( $name $($rest)*)  
    };
    // Dynamic objects don't participate in __check_event()
    ( $name:ident dyn! $([max = $max_dyn:expr])? $field_type:ident $(($($field_generic:expr),*))? $field_name:ident ; $($other_fields:tt)* ) => {
        munch_condition!( $name $($other_fields)*)
    };
    ( $name:ident dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        munch_condition!( $name $($other_fields)*)
    };
    ( $name:ident dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_condition!( $name $($other_fields)*)
    };

    ( $name:ident $([[$loop_index:expr]])? ) => {{
        String::new()
    }};

     // If encountering a for loop, repeat the body:
    ( $name:ident for($loop_left:tt <= $loop_index:ident < $loop_right:expr) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left..$loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &munch_condition!( $name [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &munch_condition!( $name $($other_fields)*);
        __s
    }};

    // Process `MATCH` tokens. Possibly inside a `for` loop 
    ( $name:ident $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident = MATCH($assert_val:expr) ; $($other_fields:tt)*  ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "__b &= ({}", stringify!($field_name));
        $( // Add loop_index as underscore
        __s += &format!("_{}", stringify!($loop_index));
        )?
        __s += &format!(" == ({}) );\n", stringify!($assert_val));
        __s += &formatt!(3; "if(!__b) {{\n");
        __s += &formatt!(4; "printerr(\"{}Event mismatch! Trying to `MATCH` in base structure: {}{}.{}", __KRED, __KMAG, stringify!($name), stringify!($field_name));
        $(
        __s += &format!("_{}", $loop_index);
        )?
        __s += &format!("{} .{}\");\n", __KRED, __KNRM);
        __s += &formatt!(4; "return 0;\n");
        __s += &formatt!(3; "}}\n"); 
        __s += &formatt!(2; "}}\n"); 
        
        __s += &munch_condition!( $name $([[$loop_index]])? $($other_fields)*);
        __s
    }};
       
    // Fields without condition block or generic'ed are skipped.
    ( $name:ident $([[$loop_index:expr]])? 
     $field_type:ident $( ( $($field_generic:expr),* ) )? $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "if(__b &= this->{}", stringify!($field_name));
        $(__s += &format!("_{}", $loop_index); )?
        __s += ".__check_event(); !__b) return __b;\n";
        __s += &munch_condition!( $name $([[$loop_index]])? $($other_fields)*);
        __s
    }};
   
    // Once the condition body is identified, go inside.
    // Bundle the $name with $field name for a unique tag.
    ( $name:ident $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident { $($inside:tt)* } ; $($other_fields:tt)*) => {{
        let mut __s = String::new();
        __s += &munch_condition_inside!( ($name,$field_name) $([[$loop_index]])? => $($inside)* );
        __s += &munch_condition!( $name $([[$loop_index]])? $($other_fields)* );
        __s
    }};
}
#[macro_export]
macro_rules! munch_condition_inside {
    // A field could encounter 5 different possible rules:
    // U32 NAMED {
    //   flag:  16 ; 18 => 0x3;    // Named range assert   (1)
    //           0 ; 15 => 0xfefe; // Unnamed range assert (2)
    //   bitflag:    20 => 0x1;    // Named bit assert     (3)
    //               19 => 0x0;    // Unnamed bit assert   (4)
    //   ENCODE(21 ; 31 => id)     // ENCODE directive     (5)
    // };
    // On either the left or the right side of => can also be the generic value (of main structure), 
    // a loop value so just paste the token and 'trust' the user it's implied somewhere.
    // Matching with a 

    // Possibility (5): skip
    ( ($name:ident, $field_name:ident) $([[$loop_index:expr]])? =>
     ENCODE( $($_tt:tt)* ) ; $($rest:tt)* ) => {
        munch_condition_inside!( ($name,$field_name) $([[$loop_index]])? => $($rest)* )
    };

    // Possibilities (1) , (2)
    ( ($name:ident, $field_name:ident) $([[$loop_index:expr]])? =>
        $(@$condition_name:ident)? $left_bound:expr ; $right_bound:expr => $assert_val:expr ; $($rest:tt)*) 
      => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "uint32_t __mask = (uint32_t)((1ull << ({} - ({}))) - 1);\n", stringify!($right_bound), stringify!($left_bound));
        __s += &formatt!(3; "uint32_t __word = (uint32_t)(this->{}", stringify!($field_name));
        $( __s += &format!("_{}", $loop_index); )?
        __s += &format!(" >> ({}));\n" , stringify!($left_bound));
        __s += &formatt!(3; "if(__b &= ((__word & __mask) == ({})); !__b) {{\n", stringify!($assert_val));
        __s += &formatt!(4; "printerr(\"{}Event mismatch! In structure: {}{}.{}" , __KRED, __KMAG, stringify!($name), stringify!($field_name));
        $( __s += &format!("_{} (inside `for`).", $loop_index); )?
        __s += &format!("{} .{}\");\n", __KRED, __KNRM);
        $( __s += &formatt!(4; "printerr(\"{} Condition name: {}{}{} .{}\");\n", __KRED, __KCYN, stringify!($condition_name), __KRED, __KNRM); )?
        __s += &formatt!(4; "printerr(\"Expected {}0x%8x{}, found: {}0x%8x{}.\\n\", {}, __word & __mask);\n", __KCYN, __KNRM, __KRED, __KNRM, stringify!($assert_val));
        __s += &formatt!(4; "return 0;\n");
        __s += &formatt!(3; "}}\n"); 
        __s += &formatt!(2; "}}\n"); 
        
        __s += &munch_condition_inside!( ($name,$field_name) $([[$loop_index]])? => $($rest)*);
        __s 
    }};
    
    // Possibilities (3), (4)
    ( ($name:ident,$field_name:ident) $([[$loop_index:expr]])? =>
        $(@$condition_name:ident)? $bit:expr => $assert_val:expr ; $($rest:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "uint32_t __word = (uint32_t)(this->{}", stringify!($field_name));
        $( __s ++ &format!("_{}", $loop_index); )?
        __s += &format!(" >> ({}));\n" , stringify!($bit));
        __s += &formatt!(3; "if(__b &= ((__word & 1) == ({})); !__b) {{\n", stringify!($assert_val));
        __s += &formatt!(4; "printerr(\"{}Event mismatch! In structure: {}{}.{}" , __KRED, __KMAG, stringify!($name), stringify!($field_name));
        $( __s += &format!("_{} (inside `for`).", $loop_index) )?
        __s += &format!("{} .{}\");\n", __KRED, __KNRM);
        $( __s += &formatt!(4; "printerr(\"{} Condition name: {}{}{} .{}\");\n", __KRED, __KCYN, stringify!($condition_name), __KRED, __KNRM); )?
        __s += &formatt!(4; "printerr(\"Expected {}0x%x{}, found: {}0x%x{}.\\n\", {}, __word & 1);\n", __KCYN, __KNRM, __KRED, __KNRM, stringify!($assert_val));
        __s += &formatt!(4; "return 0;\n");
        __s += &formatt!(3; "}}\n"); 
        __s += &formatt!(2; "}}\n"); 
        
        __s += &munch_condition_inside!( ($name,$field_name) $([[$loop_index]])? => $($rest)*);
        __s 
    }};
    
    ( ($name:ident,$field_name:ident) $([[$loop_index:expr]])? => ) => {{
        String::new()
    }};

    () => {{
        String::new()
    }}

}
