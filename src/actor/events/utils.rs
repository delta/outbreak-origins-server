// Implements a stringify function for the send event enum to be sent as a ws test message
#[macro_export]
macro_rules! enum_str {
    (enum $name:ident {
        $($eventtype:ident($value:ty)$(,)?)*
    }) => {
        pub enum $name {
            $($eventtype($value),)*
        }
        impl $name {
            pub fn stringify(&self) -> String {
                match self {
                    $($name::$eventtype(x) =>
                      serde_json::to_string(&SentEvent{event_type: stringify!($eventtype).to_string(), payload: (serde_json::to_string(x).unwrap())}).unwrap()
                      ),*
                }
            }
        }
    }
}

pub use enum_str;
