use std::fmt::Display;


#[derive(Debug)]
pub enum Locale {
    En
}

impl Display for Locale {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        type L = Locale;

        match self {
            L::En => write!(f, "en")
        }
    }
}