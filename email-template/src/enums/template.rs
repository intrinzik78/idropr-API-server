use crate::enums::Locale;

type L = Locale;

#[derive(Debug,Copy,Clone)]
pub enum Template {
    VerificationV1
}

impl Template {
    /// outputs the locale-pinned MJML template
    fn en_mjml(self) -> &'static str {
        match self {
            Self::VerificationV1 => include_str!("../templates/verification/initial_verification/v1/en.mjml")
        }
    }

    /// outputs the locale-pinned TEXT template
    fn en_text(self) -> &'static str {
        match self {
            Self::VerificationV1 => include_str!("../templates/verification/initial_verification/v1/en.text")
        }
    }

    pub fn load_mjml_template(self, locale: &Locale) -> &'static str {
        match locale {
            L::Default => Self::en_mjml(self),
            L::En => Self::en_mjml(self)
        }
    }

    pub fn load_text_template(self, locale: &Locale) -> &'static str {
        match locale {
            L::Default => Self::en_text(self),
            L::En => Self::en_text(self)
        }
    }
}