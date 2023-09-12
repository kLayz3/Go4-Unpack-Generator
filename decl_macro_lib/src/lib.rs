#[macro_export]
macro_rules! formatt {
    ($tab_count:expr; $($arg:tt)*) => {
        format!("{}","\t".repeat($tab_count)) + &format!($($arg)*)
    }
}

#[macro_export] 
macro_rules! parse_spec_file { 
    ( $($name:ident( $($generic:ident),*) { $($body:tt)* }; )* ) => {
        $( munch!($name ($($generic),*) { $($body)* } ; ) ; )*
    }
}
#[macro_export]
macro_rules! munch {
    ($name:ident( $($generic:ident),*) { $($body:tt)* };) => {
        let mut __s = String::new();
        let __file_name = format!("{}.struct", stringify!($name));
        let mut __file = File::create(&__file_name)?;
        
        __s += "#include \"go4_unpack_struct.common\"\n\n";
        __s += "template <";
        __s += &munch_generics!( $($generic),*);
        __s += &format!("typename __T = void> \nstruct __{} {{\n", stringify!($name));

        // Create fields:
        __s += &munch_members!( $($body)* );
        __s += &formatt!(1; "unordered_map<const char*, Go4UnpackPtr> m;\n\n");

        // Create a default ctor
        __s += &formatt!(1; "{}() = default;\n\n", stringify!($name));
        
        // Create the static __min_size() 
        __s += &formatt!(1; "static constexpr size_t __min_size() {{\n");
        __s += &formatt!(2; "size_t struct_size = 0;\n");
        __s += &munch_size!( $($body)* );
        __s += &formatt!(2; "return struct_size;\n");
        __s += &formatt!(1; "}}\n");
        // Create __check_event() method: 
        __s += "\n";
        __s += &formatt!(1; "void __check_event() {{\n");
        __s += &formatt!(2; "bool __b = 1;\n");
        __s += &munch_condition!( $name $($body)*);
        __s += &formatt!(2; "return __b;\n");
        __s += &formatt!(1; "}}\n\n");

        // Create __fill() method:
        __s += &formatt!(1; "void __fill(uint8_t* __event_handle, size_t& bytes_available, size_t& bytes_read) {{\n");
        __s += &formatt!(2; "bytes_read = 0;\n");
        __s += &formatt!(2; "bytes_read_sub = 0;\n");
        __s += &munch_fill!( $($body)* );
        __s += &formatt!(1; "}}\n\n"); 

        // Create __clear() method:
        __s += &formatt!(1; "void __clear() {{\n");
        // todo!()
        __s += &formatt!(1; "}}\n\n"); 

        // Create a __new() method that initializes members and calls __new() of fields
        __s += &formatt!(1; "void __new() {{\n");
        __s += &munch_encode!( $($body)* );
        __s += &formatt!(1; "}}\n\n");
        __s += &format!("}}\n");

        // Create a __min_size() method, which checks how much size does the struct have

        __s += "\n";

        __file.write(__s.as_bytes())?;
        __s.clear();
    };
    
    () => {};
}

#[macro_export]
macro_rules! munch_generics {
    ( $($generic:ident),* ) => {{
        let mut __s = String::new();
        $( __s += &format!("uint32_t {}, ", stringify!($generic)); )* 
        __s
    }};
}

#[allow(dead_code)] const MAX_DYN_DEFAULT: usize = 100;

#[macro_export]
macro_rules! munch_members {
    // Expand fields defined in a `for`  
    ( @for ( $loop_left:tt <= $loop_index:ident < $loop_right:expr ) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left .. $loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &munch_members!( [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &munch_members!( $($other_fields)*);
        __s
    }};
    
    // Dynamic fields with capacity $max_dyn hold an array. Cannot be in a `for`. 
    // Possible for structure fields without {} block or primitives with {} block
    ( dyn! $([max = $max_dyn:expr])? $field_type:ident $(($($field_generic:expr),*))? $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &format!("int N_{};\n", stringify!($field_name));
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
        __s += &munch_members!( $([[ $loop_index ]])? $( $other_fields )*);
        __s
    }};
    // Next two rules expand to the rule above.
    ( dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        munch_members!( dyn! $([max = $max_dyn])? $field_type:ident $field_name:ident ; $($other_fields:tt)* ) 
    };
    ( dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_members!( dyn! $([max = $max_dyn])? $field_type:ident $field_name:ident ; $($other_fields:tt)* ) 
    };

    // Fields with `MATCH` cannot be a generic or hold a {} block.
    ( $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_members!($([[$loop_index]])? $field_type $field_name ; $($other_fields)* )
    };

    // Generic'ed fields cannot hold condition block.
    ( $([[ $loop_index:expr ]])? $field_type:ident ( $($field_generic:expr),* ) $field_name:ident ; $($other_fields:tt)* ) => {{
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
        __s += &munch_members!( $([[ $loop_index ]])? $( $other_fields )*);
        __s
    }};

    // Condition block can only be held by a primitive
    ( $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident $( { $($condition_body:tt)* } )? ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(1; "__{}<> {}", stringify!($field_type), stringify!($field_name));
        $( // Add loop index to the name, if it is supplied
        __s += &format!("_{}", $loop_index);
        )?
        __s += ";\n";
        __s += &munch_members!( $([[ $loop_index ]])? $( $other_fields )*);
        __s   
    }};

    // At the max depth of recursion, return and pop the stack frames
    ( $([[ $loop_index:expr ]])? ) => {{
        String::new()
    }};
}
#[macro_export]
macro_rules! munch_size {
    ( @for ( $loop_left:tt <= $loop_index:ident < $loop_right:expr ) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i = $loop_left .. $loop_right {
            __s += &munch_size!( $($loop_body)*);
        }
        __s += &munch_size!( $($other_fields)*);
        __s
    }};
    // Dyn objects have minimal size 0, skip them
    ( dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        munch_size!( $($other_fields:tt)* ) 
    };
    ( dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_size!( $($other_fields:tt)* ) 
    };
    ( dyn! $([max = $max_dyn:expr])? $field_type:ident $(($($field_generic:expr),*))? $field_name:ident ; $($other_fields:tt)* ) => {
        munch_size!( $($other_fields:tt)* ) 
    };

    ( $field_type:ident $field_name:ident $( { $($condition_body:tt)* } )? ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "struct_size += __{}<>::__min_size();\n", stringify!($field_type));
        __s += &munch_size!( $($other_fields)* );
        __s
    }};

    ( $field_type:ident ( $($field_generic:expr),* ) $field_name:ident ; $($other_fields:tt)* ) => {{ 
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
        __s += &munch_size!( $($other_fields)* );
        __s
    }};

    () => {{
        String::new()
    }};
}

#[macro_export] 
macro_rules! munch_condition {
    // `$name` is the structure identifier passed to this macro by the main invocation.
    // Tokens belonging to MEMBER annotations are ignored
    ( $name:ident @MEMBER( $($__tt:tt)* ) $($rest:tt)*) => {
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
    ( $name:ident @for($loop_left:tt <= $loop_index:ident < $loop_right:expr) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
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
        __s += &munch_condition_inside!( ($name,$field_name) $([[$loop_index]])? $($inside)* );
        __s += &munch_condition!( $name $([[$loop_index]])? $($other_fields)* );
        __s
    }};
}

#[macro_export]
macro_rules! munch_condition_inside {
    // A field could encounter 5 different possible rules:
    // U32 NAMED {
    //   @flag  16 ; 18 => 0x3;    // Named range assert   (1)
    //           0 ; 15 => 0xfefe; // Unnamed range assert (2)
    //   @bitflag    20 => 0x1;    // Named bit assert     (3)
    //               19 => 0x0;    // Unnamed bit assert   (4)
    //   ENCODE(21 ; 31 => id)     // ENCODE directive     (5)
    // };
    // On either the left or the right side of => can also be the generic value (of main structure), 
    // a loop value so just paste the token and 'trust' the user it's implied somewhere.
    // Matching with a 

    // Possibility (5): skip
    ( ($name:ident, $field_name:ident) $([[$loop_index:expr]])? 
     ENCODE( $($_tt:tt)* ) ; $($rest:tt)* ) => {
        munch_condition_inside!( ($name,$field_name) $([[$loop_index]])? $($rest)* )
    };

    // Possibilities (1) , (2)
    ( ($name:ident, $field_name:ident) $([[$loop_index:expr]])?
        $(@$condition_name:ident)? $left_bound:expr ; $right_bound:expr => $assert_val:expr ; $($rest:tt)*) 
      => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "uint32_t __mask = (uint32_t)((1ull << ({} - ({}))) - 1);\n", stringify!($right_bound), stringify!($left_bound));
        __s += &formatt!(3; "uint32_t __word = (uint32_t)(this->{}", stringify!($field_name));
        $( __s ++ &format!("_{}", $loop_index); )?
        __s += &format!(" >> ({}));\n" , stringify!($left_bound));
        __s += &formatt!(3; "if(__b &= ((__word & __mask) == ({})); !__b) {{\n", stringify!($assert_val));
        __s += &formatt!(4; "printerr(\"{}Event mismatch! In structure: {}{}.{}" , __KRED, __KMAG, stringify!($name), stringify!($field_name));
        $( __s += &format!("_{} (inside `for`).", $loop_index) )?
        __s += &format!("{} .{}\");\n", __KRED, __KNRM);
        $( __s += &formatt!(4; "printerr(\"{} Condition name: {}{}{} .{}\");\n", __KRED, __KCYN, stringify!($condition_name), __KRED, __KNRM); )?
        __s += &formatt!(4; "printerr(\"Expected {}0x%8x{}, found: {}0x%8x{}.\\n\", {}, __word & __mask);\n", __KCYN, __KNRM, __KRED, __KNRM, stringify!($assert_val));
        __s += &formatt!(4; "return 0;\n");
        __s += &formatt!(3; "}}\n"); 
        __s += &formatt!(2; "}}\n"); 
        
        __s += &munch_condition_inside!( ($name,$field_name) $([[$loop_index]])? $($rest)*);
        __s 
    }};
    
    // Possibilities (3), (4)
    ( ($name:ident,$field_name:ident) $([[$loop_index:expr]])?  
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
        
        __s += &munch_condition_inside!( ($name,$field_name) $([[$loop_index]])? $($rest)*);
        __s 
    }};
    
    ( ($name:ident,$field_name:ident) $([[$loop_index:expr]])? ) => {{
        String::new()
    }}; 

}

#[macro_export]
macro_rules! munch_encode {
    // This macro will search for all ENCODE statements and create a __new() method
    // Which creates appropriate pointers to selected data words

    // Ignore MEMBER annotations. `@MEMBER` cannot be in `for` loop
    ( @MEMBER( $($_t:tt)* ) $($other:tt)* ) => {
        munch_encode!( $($other)*)
    };
     // If encountering a for loop, repeat the body:
    ( @for($loop_left:tt <= $loop_index:ident < $loop_right:expr) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left..$loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &munch_encode!( [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &munch_encode!( $($other_fields)*);
        __s
    }};

    // Fields with no condition block or generic'ed just call their own __new().
    ( $([[$loop_index:expr]])? 
     $field_type:ident $( ( $($field_generic:expr),* ) )? $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "__{}", stringify!($field_name));
        $( __s += &format!("_{}", $loop_index); )?
        __s += &format!(".__new();\n");
        __s += &munch_encode!( $([[$loop_index]])? $($other_fields)*);
        __s
    }};
    // MATCH'ed fields also can't encode anything.
    ( $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident = MATCH($match_val:expr); $($other_fields:tt)* ) => {
        munch_encode!($([[$loop_index]])? $field_type $field_name ; $($other_fields)*)
    };
    
    // Go inside the {} body where ENCODE's could live. Pass `$field_name => ` as a tag
    ( $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident { $($inside:tt)* } ; $($other_fields:tt)*) => {{
        let mut __s = String::new();
        __s += &munch_encode_inside!( $field_name $([[$loop_index]])? => $($inside)* );
        __s += &munch_encode!( $([[$loop_index]])? $($other_fields)* );
        __s
    }};

    // dyn objects without {} block don't encode anything new.
    ( dyn! $([max = $max_dyn:expr])? $field_type:ident $(($($field_generic:expr),*))? $field_name:ident ; $($other_fields:tt)* ) => {
        munch_encode!( $($other_fields)* )
    };
    ( dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_encode!( $($other_fields)* )
    };
    // Once found, go inside the {} block for `dyn!` and parse all encodes.
    ( dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($inside:tt)* } ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        let mut max_dyn: usize = MAX_DYN_DEFAULT;
        $( max_dyn = $max_dyn as usize; )?
        for(i = 0;)
        __s += &munch_encode_inside_dyn!( $field_name $(, $max_dyn)? => $($inside)* );
        __s += &munch_encode!( $($other_fields)*);
        __s
    }};

    ( $([[$loop_index:expr]])? ) => {{
        String::new()
    }};
}

#[macro_export]
macro_rules! munch_encode_inside {
    ( $field_name:ident $([[$loop_index:expr]])? =>
     ENCODE($left_bound:expr ; $right_bound:expr => $encode_id:ident $( [$encode_index:expr] )? ) ; $($rest:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3;"void* _p = (void*)&{}", stringify!($field_name));
        $(__s += &format!("_{}", $loop_index); )?
        __s += ";\n";
        __s += &formatt!(3; "Go4UnpackPtr _ptr({}, {}, _p);\n", stringify!($left_bound), stringify!($right_bound));
        __s += &formatt!(3;"m.emplace(std::make_pair(\"{}", stringify!($encode_id));
        $(__s += &format!("[{}]",stringify!($encode_index)); )?
        __s += "\", _ptr));\n";
        __s += &formatt!(2; "}}\n");
        __s += &munch_encode_inside!( $field_name $([[$loop_index]])? => $($rest)*);
        __s
    }};
    // Ignore condition statements (1,2)
    ( $field_name:ident $([[$loop_index:expr]])? =>
        $(@$condition_name:ident)? $left_bound:expr ; $right_bound:expr => $assert_val:expr ; $($rest:tt)*) => {
        munch_encode_inside!( $field_name $([[$loop_index:expr]])? => $($rest)* )
    }; 
    // Ignore condition statements (3,4)
    ( $field_name:ident $([[$loop_index:expr]])? =>
        $(@$condition_name:ident)? $bit:expr => $assert_val:expr ; $($rest:tt)* ) => { 
        munch_encode_inside!( $field_name $([[$loop_index:expr]])? => $($rest)*)
    };
    ( $field_name:ident $([[$loop_index:expr]])? => ) => {{
        String::new()
    }};
}

#[macro_export]
macro_rules! munch_encode_inside_dyn {
    // Dynamic Fields:
    ( $field_name:ident $(, $max_dyn:expr )? =>
     ENCODE($left_bound:expr ; $right_bound:expr => $encode_id:ident ) ; $($rest:tt)* ) => {{
        let mut __s = String::new();
        let mut max_dyn: usize = MAX_DYN_DEFAULT;
        $( max_dyn = $max_dyn as usize; )?
         __s += &formatt!(2; "for(int i = 0; i < {}; ++i) {{\n", max_dyn);
        __s += &formatt!(3;"void* _p = (void*)&{}[i];\n", stringify!($field_name));
        __s += &formatt!(3; "Go4UnpackPtr _ptr({}, {}, _p);\n", stringify!($left_bound), stringify!($right_bound));
        __s += &formatt!(3;"m.emplace(std::make_pair(\"{}[i]\", _ptr));\n", stringify!($encode_id));
        __s += &formatt!(2; "}}\n");
        __s += &munch_encode_inside!( $field_name $(, $max_dyn )? => $($rest)*);
        __s
    }};
    ( $field_name:ident $(, $max_dyn:expr )? =>
        $(@$condition_name:ident)? $left_bound:expr ; $right_bound:expr => $assert_val:expr ; $($rest:tt)*) => {
        munch_encode_inside!( $field_name $(, $max_dyn )? => $($rest)*)
    }; 
    ( $field_name:ident $(, $max_dyn:expr )? =>
     $(@$condition_name:ident)? $bit:expr => $assert_val:expr ; $($rest:tt)* ) => { 
        munch_encode_inside!( $field_name $(, $max_dyn )? => $($rest)*)
    };
    ( $field_name:ident $(, $max_dyn:expr )? => ) => {{
        String::new()
    }};
}

#[macro_export] 
macro_rules! munch_fill { 
    ( @for ( $loop_left:tt <= $loop_index:ident < $loop_right:expr ) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left .. $loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &munch_fill!( [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &munch_fill!( $($other_fields)*);
        __s
    }};

    // If encountering either a primitive or composed - just call their __fill()
    ( $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{}", stringify!($field_name));
        $( __s += &format!("_{}", $loop_index); )?
        __s += &format!(".__fill(__event_handle, bytes_read_sub);\n");
        __s += &formatt!(2;"__event_handle += bytes_read_sub;\n");
        __s += &formatt!(2;"bytes_read += bytes_read_sub;\n\n");
        __s += &munch_fill!( $($other_fields)*);
        __s
    }}; 
    // Following three rules expand to the rule above:
    ( $([[ $loop_index:expr ]])? $field_type:ident ( $($field_generic:expr),* ) $field_name:ident ; $($other_fields:tt)* ) => {
        munch_fill!( $([[$loop_index]])? $field_type $field_name ; $($other_fields)* )
    };
    ( $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        munch_fill!( $([[$loop_index]])? $field_type $field_name ; $($other_fields)* )
    };
    ( $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_fill!( $([[$loop_index]])? $field_type $field_name ; $($other_fields)* )
    };
    
    // dyn! fields keep filling the array until either array is full, buffer is over or condition
    // is violated.
    ( dyn! $([max = $max_dyn:expr])? $field_type:ident $(($($field_generic:expr),*))? $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
    }};
    ( dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident { $($condition_body:tt)* } ; $($other_fields:tt)* ) => {
        munch_condition!( $name $($other_fields)*)
    };
    ( dyn! $([max = $max_dyn:expr])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {
        munch_condition!( $name $($other_fields)*)
    };

    

    ( $([[ $loop_index:expr ]])? ) => {{
        String::new()
    }};

}
