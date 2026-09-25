#[macro_export]
macro_rules! sql {
    ($file:expr) => {
        include_str!(concat!(
            "../../../ressources/turso/queries/",
            stringify!($file),
            ".sql"
        ))
    };
}
