#[macro_export]
macro_rules! debug_from_display {
    ($name:ident) => {
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                (self as &dyn std::fmt::Display).fmt(f)
            }
        }
    };
}
