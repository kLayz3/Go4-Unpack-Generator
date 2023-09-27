#[macro_export]
macro_rules! formatt {
    ($tab_count:expr; $($arg:tt)*) => {
        format!("{}","\t".repeat($tab_count)) + &format!($($arg)*)
    }
}

#[macro_export] 
macro_rules! parse_spec_file { 
    ( $($name:ident( $($generic:ident),*) { $($body:tt)* }; )*
      $(SUBEVENT!($subev_name:ident) { $($subev_body:tt)* }; )* 
      $(EVENT { $($ev_body:tt)* }; )? ) 
        => {
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
        __s += &munch_fields!(@$($body)* );
        __s += &formatt!(1; "unordered_map<const char*, Go4UnpackPtr> m;\n\n");

        // Create a default ctor
        __s += &formatt!(1; "{}() = default;\n\n", stringify!($name));

        // Create an __init() method that initializes members and calls __init() of fields 
        // without a `{}` block
        __s += &formatt!(1; "void __init() {{\n");
        __s += &munch_encode!(@$($body)* );
        __s += &formatt!(1; "}}\n\n");
        __s += &format!("}}\n");
 
        // Create the static __min_size() 
        __s += &formatt!(1; "static constexpr size_t __min_size() {{\n");
        __s += &formatt!(2; "size_t struct_size = 0;\n");
        __s += &munch_size!(@$($body)* );
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
        __s += &formatt!(2; "if (__min_size() > bytes_available) throw std::runtime_error(\"Subevent boundary reached. Cannot read anymore.\");\n");
        __s += &formatt!(2; "bytes_read = 0;\n");
        __s += &formatt!(2; "size_t bytes_read_sub = 0;\n");
        __s += &munch_fill!(@$($body)* );
        __s += &formatt!(1; "}}\n\n"); 

        // Create __clear() method:
        __s += &formatt!(1; "void __clear() {{\n");
        __s += &munch_clear!(@$($body)*);
        __s += &formatt!(1; "}}\n\n"); 
       
        __s += "\n";

        __file.write(__s.as_bytes())?;
        __s.clear();
    };

    // Ignore subevent and event tokens
    (SUBEVENT(name = $name:ident) { $($body:tt)* };) => {
        // there should not be any encodes or struct definitions here.
        // Here shall be only pure declarations
        // in the format: 
        // `` name = type ; ``
        //
        // todo!
    };
    (EVENT { $($body:tt)* };) => {
        // there should not be any encodes or struct definitions here.
        // Here are only subevent names with one specific format:
        // `` name = subevent_name(type=T, subtype=ST, \
        // control=CT, procid=procID, crate=CR, subcrate=subCR); ``
        
        // subevent specifiers can be ommited as long as different specified subevents are
        // distincted in the LMD data, else it shall fail.
        //
        // todo!
    };
    () => {};
}

include!("parse_generics.rs");
include!("parse_fields.rs");
include!("parse_size.rs");
include!("parse_condition.rs");
include!("parse_encode.rs");
include!("parse_fill.rs");
include!("parse_clear.rs");
