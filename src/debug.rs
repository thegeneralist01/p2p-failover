use std::sync::OnceLock;

static DEBUG_ENABLED: OnceLock<bool> = OnceLock::new();

pub fn is_debug_enabled() -> bool {
    *DEBUG_ENABLED.get_or_init(|| {
        std::env::var("DEBUG")
            .map(|val| val == "1" || val.to_lowercase() == "true")
            .unwrap_or(false)
    })
}

#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        if $crate::debug::is_debug_enabled() {
            println!($($arg)*);
        }
    };
}
