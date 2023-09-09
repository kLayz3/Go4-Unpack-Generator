#[macro_export]
macro_rules! test_lex {
    ($x:ident) => {
        println!("Found id!~ : {}", stringify!($x));
    };
    () => {};
}
#[macro_export]
macro_rules! formatt {
    ($tab_count:expr; $($arg:tt)*) => {
        format!("{}","\t".repeat($tab_count)) + &format!($($arg)*)
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
        __s += &munch!(@munch_generics $($generic),*);
        __s += &format!("typename __T = void> \nstruct __{} {{\n", stringify!($name));

        // Create fields:
        __s += &munch!(@munch_members $($body)*);
        __s += &formatt!(1; "unordered_map<const char*, Go4UnpackPtr> m;\n");

        // Create a default ctor
        __s += "\n";
        __s += &formatt!(1; "{}() = default;\n", stringify!($name));
        // Create __check_event() method: 
        __s += "\n";
        __s += &formatt!(1; "void __check_event() {{\n");
        __s += &formatt!(2; "bool __b = 1;\n");
        __s += &munch!(@munch_condition $name $($body)*);
        __s += &formatt!(2; "return __b;\n");
        __s += &formatt!(1; "}}\n\n");

        // Create __fill() method:
        __s += &formatt!(1; "void __fill(uint8_t* __event_handle, size_t& bytes_read) {{\n");
        __s += &formatt!(2; "bytes_read = 0;\n");
        // todo!()
        __s += &formatt!(1; "}}\n\n"); 

        // Create __clear() method:
        __s += &formatt!(1; "void __clear() {{\n");
        // todo!()
        __s += &formatt!(1; "}}\n\n"); 

        // Create a __new() method that initializes members and calls __new() of fields
        __s += &formatt!(1; "void __new() {{\n");
        __s += &munch!(@munch_encode $($body)*);
        __s += &formatt!(1; "}}\n\n");
        __s += &format!("}}\n");

        __s += "\n";

        /*
        // Make a new struct with final size_t _N template param,
        //in case it's used as a further member:
        __s += &format!("\ntemplate <");
        __s += &munch!(@munch_generics $($generic),*);
        __s += "size_t N = 1, typename __T = void>\n";
        __s += &format!("struct __{} {{\n", stringify!($name));
        __s += &formatt!(1; "{}<", stringify!($name));
        $( __s += &format!("{},", stringify!($generic)); )*
        __s += "void> x[N];\n\n";
        __s += &formatt!(1; "__{}() = default;\n\n", stringify!($name));
        __s += &formatt!(1; "inline bool __check_event() {{\n");
        __s += &formatt!(2; "bool b = 1;\n");
        __s += &formatt!(2; "for(auto& _x : x) {{b &= _x.__check_event();}};\n");
        __s += &formatt!(2; "return b;\n");
        __s += &formatt!(1; "}}\n");
        __s += &formatt!(1; "inline void __fill(uint8_t* __event_handle, size_t& bytes_read) {{\n");
        __s += &formatt!(2; "size_t b = 0; bytes_read = 0;\n");
        __s += &formatt!(2; "for(auto& _x : x) {{\n");
        __s += &formatt!(3; "_x.__fill(__event_handle, b);\n");
        __s += &formatt!(3; "__event_handle += b;\n");
        __s += &formatt!(3; "__bytes_read += b;\n");
        __s += &formatt!(2; "}}\n"); 
        __s += &formatt!(1; "}}\n"); 
        __s += &formatt!(1; "inline bool __clear() {{\n");
        __s += &formatt!(2; "for(auto& _x : x) {{_x.__clear();}}\n");
        __s += &formatt!(1; "}}\n");
        __s += &formatt!(1; "inline void __new() {{\n");
        __s += &formatt!(2; "for(auto& _x : x) {{_x.__new();}}\n");
        __s += &formatt!(1; "}}\n");
        __s += &formatt!(0; "}}\n");
        */
        __file.write(__s.as_bytes())?;
        __s.clear();
    };
    
    // Rule to lex generic's
    (@munch_generics $($generic:ident),* ) => {{
        let mut __s = String::new();
        $( __s += &format!("uint32_t {}, ", stringify!($generic)); )* 
        __s
    }};

    // Ignore if encountering a MEMBER:
    (@munch_members @MEMBER( $($_t:tt)* ) $($other:tt)* ) => {
        munch!(@munch_members $($other)*)
    };

    // Rule to expand members defined in a `for`  
    (@munch_members for ( $loop_left:tt <= $loop_index:ident < $loop_right:expr ) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left .. $loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &munch!(@munch_members [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &munch!(@munch_members $($other_fields)*);
        __s
    }};
    
    // Members with `MATCH` cannot be a generic or hold a {} block.
    (@munch_members $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident = MATCH($field_val:expr) ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(1; "__{}<> {}", stringify!($field_type), stringify!($field_name));
        $(
        __s += &format!("_{}", $loop_index);
        )?
        __s += ";\n";
        __s += &munch!(@munch_members $([[$loop_index]])? $($other_fields)*);
        __s
    }};
    // Generic'ed members cannot hold condition block.
    (@munch_members $([[ $loop_index:expr ]])? $field_type:ident ( $($field_generic:tt),* ) $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        let mut generics_added = 0;
        __s += &formatt!(1; "__{}", stringify!($field_type));
        __s += "<";
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
        __s += &munch!(@munch_members $([[ $loop_index ]])? $( $other_fields )*);
        __s
    }};
    // Condition block can only be held by primitives
    (@munch_members $([[ $loop_index:expr ]])? $field_type:ident $field_name:ident $( { $($condition_body:tt)* } )? ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(1; "__{}<> {}", stringify!($field_type), stringify!($field_name));
        $( // Add loop index to the name, if it is supplied
        __s += &format!("_{}", $loop_index);
        )?
        __s += ";\n";
        __s += &munch!(@munch_members $([[ $loop_index ]])? $( $other_fields )*);
        __s   
    }};
    // At the max depth of recursion, return blank string
    (@munch_members $([[ $loop_index:expr ]])? ) => {{
        String::new()
    }};

    // Rules to lex __check_event:
    // Tokens belonging to MEMBER annotations are ignored
    (@munch_condition $name:ident @MEMBER( $($__tt:tt)* ) $($rest:tt)*) => {
        munch!(@munch_condition $name $($rest)*)  
    };
    (@munch_condition $name:ident $([[$loop_index:expr]])?) => {{
        String::new()
    }};
     // If encountering a for loop, repeat the body:
    (@munch_condition $name:ident for($loop_left:tt <= $loop_index:ident < $loop_right:expr) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left..$loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &munch!(@munch_condition $name [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &munch!(@munch_condition $name $($other_fields)*);
        __s
    }};

    // Process `MATCH` tokens. Possibly inside a `for` loop 
    (@munch_condition $name:ident $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident = MATCH($assert_val:expr) ; $($other_fields:tt)*  ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "__b &= ({}", stringify!($field_name));
        $( // Add loop_index as underscore
        __s += &format!("_{}", stringify!($loop_index));
        )?
        __s += &format!(" == {});\n", stringify!($assert_val));
        __s += &formatt!(3; "if(!__b) {{\n");
        __s += &formatt!(4; "printerr(\"{}Event mismatch! Trying to `MATCH` in base structure: {}{}.{}", __KRED, __KMAG, stringify!($name), stringify!($field_name));
        $(
        __s += &format!("_{}", $loop_index);
        )?
        __s += &format!("{} .{}\");\n", __KRED, __KNRM);
        __s += &formatt!(4; "return 0;\n");
        __s += &formatt!(3; "}}\n"); 
        __s += &formatt!(2; "}}\n"); 
        
        __s += &munch!(@munch_condition $name $($other_fields)*);
        __s 
    }};
       
    // Fields with no condition block, or generic'ed are skipped.
    (@munch_condition $name:ident $([[$loop_index:expr]])? 
     $field_type:ident $( ( $($field_generic:tt),* ) )? $field_name:ident ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "__b &= this->{}", stringify!($field_name));
        $(__s += &format!("_{}", $loop_index); )?
        __s += ".check_event();\n";
        __s += &munch!(@munch_condition $name $([[$loop_index]])? $($other_fields)*);
        __s
    }};
   
    // Go inside the condition body
    (@munch_condition $name:ident $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident { $($inside:tt)* } ; $($other_fields:tt)*) => {{
        let mut __s = String::new();
        __s += &munch!(@munch_condition_inside ($name,$field_name) $([[$loop_index]])? $($inside)* );
        __s += &munch!(@munch_condition $name $([[$loop_index]])? $($other_fields)* );
        __s
    }};

    // Generally a field could encounter 5 different possible rules:
    // U32 NAMED {
    //   @flag  16..18 => 0x3;    // Named range assert   (1)
    //           0..15 => 0xbebe; // Unnamed range assert (2)
    //   @bitflag   20 => 0x1;    // Named bit assert     (3)
    //              19 => 0x0;    // Unnamed bit assert   (4)
    //   ENCODE(21..31 => id)     // ENCODE directive     (5)
    // };
    // On either the left or the right side of => can also be the generic value, loop value
    // so just paste the token and 'trust' the user it's implied somewhere.

    (@munch_condition_inside ($name:ident,$field_name:ident) $([[$loop_index:expr]])?) => {{
        String::new()
    }}; 

    // Possibility (5): skip
    (@munch_condition_inside ($name:ident, $field_name:ident) $([[$loop_index:expr]])? 
     ENCODE( $($_tt:tt)* ) ; $($rest:tt)* ) => {
        munch!(@munch_condition_inside ($name,$field_name) $([[$loop_index]])? $($rest)* )
    };

    // Possibility (1)
    (@munch_condition_inside ($name:ident, $field_name:ident) $([[$loop_index:expr]])?  
        @$condition_name:ident $left_bound:tt .. $right_bound:tt => $assert_val:tt ; $($rest:tt)*) 
      => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "uint32_t __mask = (uint32_t)((1ull << ({} - {})) - 1);\n", $right_bound, $left_bound);
        __s += &formatt!(3; "uint32_t __word = (uint32_t)(this->{}", stringify!($field_name));
        $( __s ++ &format!("_{}", $loop_index); )?
        __s += &format!(" >> {});\n" , stringify!($left_bound));
        __s += &formatt!(3; "if(__b &= ((__word & __mask) == {}); !b) {{\n", stringify!($assert_val));
        __s += &formatt!(4; "printerr(\"{}Event mismatch! In structure: {}{}.{}" , __KRED, __KMAG, stringify!($name), stringify!($field_name));
        $( __s += &format!("_{}", $loop_index) )?
        __s += &format!("{} .{}\");\n", __KRED, __KNRM);
        __s += &formatt!(4; "printerr(\"{} Condition name: {}{}{} .{}\");\n", __KRED, __KCYN, stringify!($condition_name), __KRED, __KNRM);
        __s += &formatt!(4; "printerr(\"Expected {}0x%8x{}, found: {}0x%8x{}.\\n\", {}, __word & __mask);\n", __KCYN, __KNRM, __KRED, __KNRM, stringify!($assert_val));
        __s += &formatt!(4; "return 0;\n");
        __s += &formatt!(3; "}}\n"); 
        __s += &formatt!(2; "}}\n"); 
        
        __s += &munch!(@munch_condition_inside ($name,$field_name) $([[$loop_index]])? $($rest)*);
        __s 
    }};
    
    // Possibility (2)
    (@munch_condition_inside ($name:ident, $field_name:ident) $([[$loop_index:expr]])?  
        $left_bound:tt .. $right_bound:tt => $assert_val:tt ; $($rest:tt)*) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "uint32_t __mask = (uint32_t)((1ull << ({} - {})) - 1);\n", $right_bound, $left_bound);
        __s += &formatt!(3; "uint32_t __word = (uint32_t)(this->{}", stringify!($field_name));
        $( __s ++ &format!("_{}", $loop_index); )?
        __s += &format!(" >> {});\n" , stringify!($left_bound));
        __s += &formatt!(3; "if(__b &= ((__word & __mask) == {}); !b) {{\n", stringify!($assert_val));
        __s += &formatt!(4; "printerr(\"{}Event mismatch! In structure: {}{}.{}" , __KRED, __KMAG, stringify!($name), stringify!($field_name));
        $( __s += &format!("_{}", $loop_index) )?
        __s += &format!("{} .{}\");\n", __KRED, __KNRM);
        __s += &formatt!(4; "printerr(\"Expected {}0x%8x{}, found: {}0x%8x{}.\\n\", {}, __word & __mask);\n", __KCYN, __KNRM, __KRED, __KNRM, stringify!($assert_val));
        __s += &formatt!(4; "return 0;\n");
        __s += &formatt!(3; "}}\n"); 
        __s += &formatt!(2; "}}\n"); 
        
        __s += &munch!(@munch_condition_inside ($name,$field_name) $([[$loop_index]])? $($rest)*);
        __s 
    }};

    // Possibility (3)
    (@munch_condition_inside ($name:ident,$field_name:ident) $([[$loop_index:expr]])?  
        @$condition_name:ident $bit:tt => $assert_val:tt ; $($rest:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "uint32_t __word = (uint32_t)(this->{}", stringify!($field_name));
        $( __s ++ &format!("_{}", $loop_index); )?
        __s += &format!(" >> {});\n" , stringify!($bit));
        __s += &formatt!(3; "if(__b &= ((__word & 1) == {}); !b) {{\n", stringify!($assert_val));
        __s += &formatt!(4; "printerr(\"{}Event mismatch! In structure: {}{}.{}" , __KRED, __KMAG, stringify!($name), stringify!($field_name));
        $( __s += &format!("_{}", $loop_index) )?
        __s += &format!("{} .{}\");\n", __KRED, __KNRM);
        __s += &formatt!(4; "printerr(\"{} Condition name: {}{}{} .{}\");\n", __KRED, __KCYN, stringify!($condition_name), __KRED, __KNRM);
        __s += &formatt!(4; "printerr(\"Expected {}0x%x{}, found: {}0x%x{}.\\n\", {}, __word & 1);\n", __KCYN, __KNRM, __KRED, __KNRM, stringify!($assert_val));
        __s += &formatt!(4; "return 0;\n");
        __s += &formatt!(3; "}}\n"); 
        __s += &formatt!(2; "}}\n"); 
        
        __s += &munch!(@munch_condition_inside ($name,$field_name) $([[$loop_index]])? $($rest)*);
        __s 
    }};

    // Possibility (4)
    (@munch_condition_inside ($name:ident,$field_name:ident) $([[$loop_index:expr]])?  
        $bit:tt => $assert_val:tt ; $($rest:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3; "uint32_t __word = (uint32_t)(this->{}", stringify!($field_name));
        $( __s ++ &format!("_{}", $loop_index); )?
        __s += &format!(" >> {});\n" , stringify!($bit));
        __s += &formatt!(3; "if(__b &= ((__word & 1) == {}); !b) {{\n", stringify!($assert_val));
        __s += &formatt!(4; "printerr(\"{}Event mismatch! In structure: {}{}.{}" , __KRED, __KMAG, stringify!($name), stringify!($field_name));
        $( __s += &format!("_{}", $loop_index) )?
        __s += &format!("{} .{}\");\n", __KRED, __KNRM);
        __s += &formatt!(4; "printerr(\"Expected {}0x%x{}, found: {}0x%x{}.\\n\", {}, __word & 1);\n", __KCYN, __KNRM, __KRED, __KNRM, stringify!($assert_val));
        __s += &formatt!(4; "return 0;\n");
        __s += &formatt!(3; "}}\n"); 
        __s += &formatt!(2; "}}\n"); 
        
        __s += &munch!(@munch_condition_inside ($name,$field_name) $([[$loop_index]])? $($rest)*);
        __s 
    }};
    
    // Search for all ENCODE statements
    (@munch_encode $([[$loop_index:expr]])?) => {{
        String::new()
    }};

    // Ignore MEMBER annotations
    (@munch_encode @MEMBER( $($_t:tt)* ) $($other:tt)* ) => {
        munch!(@munch_encode $($other)*)    
    };
     // If encountering a for loop, repeat the body:
    (@munch_encode for($loop_left:tt <= $loop_index:ident < $loop_right:expr) { $($loop_body:tt)* } $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        for i in $loop_left..$loop_right {
            __s += &format!("#define {} {}\n", stringify!($loop_index), i);
            __s += &munch!(@munch_encode [[i]] $($loop_body)*);
            __s += &format!("#undef {}\n", stringify!($loop_index));
        }
        __s += &munch!(@munch_encode $($other_fields)*);
        __s
    }};

    // Fields with no condition block; array'ed or generic'ed just call their own __new().
    (@munch_encode $([[$loop_index:expr]])? 
     $field_type:ident $( ( $($field_generic:tt),* ) )? $field_name:ident $( [$array_size:expr] )? ; $($other_fields:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "__{}", stringify!($field_name));
        $( __s += &format!("_{}", $loop_index); )?
        __s += &format!(".__new();\n");
        __s += &munch!(@munch_encode $([[$loop_index]])? $($other_fields)*);
        __s
    }};

    // Fields with {} block cannot be array'ed or generic'ed, but could be in a `for` loop
    // Go inside the {} body:
    (@munch_encode $([[$loop_index:expr]])? 
     $field_type:ident $field_name:ident { $($inside:tt)* } ; $($other_fields:tt)*) => {{
        let mut __s = String::new();
        __s += &munch!(@munch_encode_inside $field_name $([[$loop_index]])? => $($inside)* );
        __s += &munch!(@munch_encode $([[$loop_index]])? $($other_fields)* );
        __s
    }};

    
    (@munch_encode_inside $field_name:ident $([[$loop_index:expr]])? =>
     ENCODE($left_bound:tt .. $right_bound:tt => $encode_id:ident $( [$encode_index:expr] )? ) ; $($rest:tt)* ) => {{
        let mut __s = String::new();
        __s += &formatt!(2; "{{\n");
        __s += &formatt!(3;"void* _p = (void*)&{}", stringify!($field_name));
        $(__s += &format!("_{}", $loop_index); )?
        __s += ";\n";
        __s += &formatt!(3; "Go4UnpackPtr _ptr({}, {}, _p);\n", stringify!($left_bound), stringify!($right_bound));
        __s += &formatt!(3;"m.emplace(std::make_pair(\"{}", stringify!($encode_id));
        $(__s += &format!("[{}]",stringify!($encode_index)); )?
        __s += "\", _p));\n";
        __s += &formatt!(2; "}}\n");
        __s += &munch!(@munch_encode_inside $field_name $([[$loop_index]])? => $($rest)*);
        __s
    }};
    
    // Ignore condition statements (1)
    (@munch_encode_inside $field_name:ident $([[$loop_index:expr]])? =>
        @$condition_name:ident $left_bound:tt .. $right_bound:tt => $assert_val:tt ; $($rest:tt)*) => {
        munch!(@munch_encode_inside $field_name $([[$loop_index:expr]])? => $($rest)* )
    }; // Ignore condition statement (2)
    (@munch_encode_inside $field_name:ident $([[$loop_index:expr]])? => 
        $left_bound:tt .. $right_bound:tt => $assert_val:tt ; $($rest:tt)*) => {
        munch!(@munch_encode_inside $field_name $([[$loop_index]])? => $($rest)*)
    }; // Ignore condition statements (4)
    (@munch_encode_inside $field_name:ident $([[$loop_index:expr]])? =>
        @$condition_name:ident $bit:tt => $assert_val:tt ; $($rest:tt)* ) => { 
        munch!(@munch_encode_inside $field_name $([[$loop_index:expr]])? => $($rest)*)
    }; // Ignore condition statements (5)
    (@munch_encode_inside $field_name:ident $([[$loop_index:expr]])? => 
        $bit:tt => $assert_val:tt ; $($rest:tt)* ) => { 
        munch!(@munch_encode_inside $field_name $([[$loop_index:expr]])? => $($rest)*)
    };
    (@munch_encode_inside $field_name:ident $([[$loop_index:expr]])? => ) => {{
        String::new()
    }};

    () => {};
}
