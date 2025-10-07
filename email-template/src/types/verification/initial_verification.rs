use std::collections::BTreeMap;
use crate::{traits::{EmailTemplate, ToVars}, enums::Template};

pub struct InitialVerificationEmail<'a> {
    pub verify_url: &'a str,
    pub expire_minutes: u32
}


impl<'a> ToVars for InitialVerificationEmail<'a> {
    fn to_vars(&self) -> BTreeMap<&'static str, String> {
        let mut m = BTreeMap::new();
        m.insert("verify_url", self.verify_url.to_string());
        m.insert("expire_minutes", self.expire_minutes.to_string());

        m
    }
}

impl<'a> EmailTemplate for InitialVerificationEmail<'a> {
    fn variant() -> Template { Template::VerificationV1 }
}



#[cfg(test)]
mod tests {
    use crate::enums::Locale;

    use super::*;

    #[test]
    fn render_email_template() {
        let url = String::from("this is the url");
        let expire_minutes = 10;
        let t = InitialVerificationEmail {
            verify_url: &url,
            expire_minutes
        };

        let locale = Locale::En;
        let rendered = t.render(&locale).unwrap();

        println!("{}", rendered.html);
    }
}