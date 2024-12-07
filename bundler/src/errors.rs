use std::{borrow::Cow, fmt::Display};

use anyhow::Error;
use colored::Colorize;
#[derive(Debug)]
struct CustomError {
    class: &'static str,
    message: Cow<'static, str>,
}
impl CustomError {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(class: &'static str, message: impl Into<Cow<'static, str>>) -> Error {
        CustomError {
            class,
            message: message.into(),
        }
        .into()
    }
}
impl std::error::Error for CustomError {}
impl Display for CustomError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.class.red().bold(), self.message)
    }
}
pub fn generic_error(message: impl Into<Cow<'static, str>>) -> Error {
    CustomError::new("Error", message)
}
