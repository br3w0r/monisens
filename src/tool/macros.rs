#[macro_export]
macro_rules! debug_from_display {
    ($name:ident) => {
        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                (self as &dyn fmt::Display).fmt(f)
            }
        }
    }
}
