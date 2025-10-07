use derive_more::derive::From;
// use std::fmt::Display;
use std::io;

// type T = TemplateError;

#[repr(u32)]
#[derive(Debug,From)]
pub enum TemplateError {
    #[from]
    FSError(io::Error),

    #[from]
    MrmlParse(mrml::prelude::parser::Error),
    
    #[from]
    MrmlRender(mrml::prelude::render::Error),

    // define custom errors default ↴
    Generic(String),
    MissingTagTermination,
    MissingVar(String),
    UnfilledPlaceholders,
    UnknownVar(String),

    // disabled by default ↴
    // DevError(String),
}

// impl std::error::Error for TemplateError {}

// impl Display for TemplateError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         match self {
//             // T::SendingEmailAddressNotPermitted => write!(f, "email not on the permitted list of senders found in the .env file"),
//             _ => write!(f, "{self:?}")
//         }
//     }
// }
