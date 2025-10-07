use std::collections::BTreeMap;

use mrml::prelude::render::RenderOptions;

use crate::{
    enums::{Extension,Locale,Template,TemplateError},
    types::RenderedEmail
};

type Result<T> = std::result::Result<T,TemplateError>;

pub struct RenderEngine;

impl RenderEngine {
    fn html_escape(s: &str) -> String {
        let mut escaped = String::with_capacity(s.len());

        for ch in s.chars() {
            match ch {
                '&' => escaped.push_str("&amp;"),
                '<' => escaped.push_str("&lt;"),
                '>' => escaped.push_str("&gt;"),
                '"' => escaped.push_str("&quot;"),
                '\''=> escaped.push_str("&#39;"),
                _ =>   escaped.push(ch),
            }
        }
        
        escaped
    }

    /// ren
    /// {{key}} = escaped ; {{{key}}} = raw
    fn render_strict(template: &str, vars: &BTreeMap<&str, String>) -> Result<String> {
        // check for unknown keys
        for k in vars.keys() {
            let a = format!("{{{{{k}}}}}");
            let b = format!("{{{{{{{k}}}}}}}");

            if !template.contains(&a) && !template.contains(&b) {
                return Err(TemplateError::UnknownVar(k.to_string()));
            }
        }

        // convert template to bytes for processing
        let bytes = template.as_bytes();
        let eof = bytes.len();

        // build the output buf
        let mut out = String::with_capacity(template.len());

        let mut idx = 0;

        while idx < bytes.len() {
            if idx+1 < eof && bytes[idx] == b'{' && bytes[idx+1] == b'{' {
                let raw = idx+2 < eof && bytes[idx+2] == b'{';
                let open = if raw { 3 } else { 2 };
                let start = idx + open;
                let close = if raw { "}}}" } else { "}}" };
                
                if let Some(end) = template[start..].find(close) {
                    let key = template[start..start+end].trim();
                    let val = vars.get(key).ok_or_else(|| TemplateError::MissingVar(key.into()))?;

                    if raw {
                        out.push_str(val);
                    } else {
                        let escaped = Self::html_escape(val);
                        out.push_str(&escaped);
                    }

                    idx = start + end + close.len();
                    continue;
                } else {
                    return Err(TemplateError::MissingTagTermination);
                }
            }

            out.push(bytes[idx] as char);

            idx += 1;
        }
        
        // check for missed replacements
        if out.contains("{{") {
            return Err(TemplateError::UnfilledPlaceholders);
        }

        Ok(out)
    }

    /// loads raw template str from memory
    fn load_template(template: &Template, locale: &Locale, extension: Extension) -> &'static str {
        match extension {
            Extension::Mjml => template.load_mjml_template(locale),
            Extension::Text  => template.load_text_template(locale)
        }
    }

    /// main entry point to render
    pub fn render(template: Template, locale: &Locale, vars: &BTreeMap<&str, String>) -> Result<RenderedEmail> {
        // load templates
        let mjml_string = Self::load_template(&template, locale, Extension::Mjml);
        let text_string = Self::load_template(&template, locale, Extension::Text);

        // preprocess to verify strict handling of dynamic content
        let pre_processed= Self::render_strict(&mjml_string, vars)?;
        let text= Self::render_strict(&text_string, vars)?;
        
        // define rendering options
        let mut opts = RenderOptions::default();
        opts.disable_comments = true;
        
        // render mjml to html
        let html = mrml::parse(&pre_processed)?
            .render(&opts)?;

        Ok(RenderedEmail { html, text })
    }
}