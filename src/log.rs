use std::sync::OnceLock;

static VERBOSE_ENABLED: OnceLock<bool> = OnceLock::new();

pub fn is_verbose_enabled() -> bool {
    *VERBOSE_ENABLED.get_or_init(|| {
        std::env::var("VERBOSE")
            .map(|val| val == "1" || val.to_lowercase() == "true")
            .unwrap_or(
                // Verbose by default when debugging
                std::env::var("DEBUG")
                    .map(|val| val == "1" || val.to_lowercase() == "true")
                    .unwrap_or(false),
            )
    })
}

#[macro_export]
macro_rules! log {
    ($($arg:tt)*) => {
        if $crate::log::is_verbose_enabled() {
            println!($($arg)*);
        }
    };
}
