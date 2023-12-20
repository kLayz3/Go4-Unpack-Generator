#[macro_export]
macro_rules! parse_event { 
    // Ignore subevent and event tokens
    ( [ $($ev_headers:ident = $val:expr),* ] $($event_body)* ) => {{
        // Here shall be only pure instantizations of subevents, no `for` loops.
        // in the format of composite types:
        // subev_name = subev_type()
                
    }};
}

