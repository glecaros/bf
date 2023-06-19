#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl<T: std::fmt::Display> From<T> for Error {
    fn from(err: T) -> Self {
        Error {
            message: format!("{}", err),
        }
    }
}

#[macro_export]
macro_rules! internal_error {
    ($($arg:tt)*) => {{
        let message = format!($($arg)+);
        let error = crate::error::Error::from(message);
        error
    }}
}
