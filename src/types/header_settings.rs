use actix_cors::Cors;
use actix_web::{
    dev::RequestHead,
    http,
    http::header::{
        HeaderValue
    }
};
pub struct HeaderSettings;

impl HeaderSettings {
    pub fn dev_cors() -> Cors {
        let headers = vec![http::header::AUTHORIZATION, http::header::ACCEPT, http::header::CONTENT_TYPE];
        let methods  = ["GET", "POST", "OPTIONS"];

        Cors::default()
            .allow_any_origin()
            .allowed_methods(methods)
            .allowed_headers(headers)
            .max_age(3600)
    }

    pub fn filter_origin(header: &HeaderValue, request: &RequestHead) -> bool {
        println!("header: {:?}", header);
        println!("request header: {:?}", request);
        true
    }

    pub fn prod_cors() -> Cors {
        let headers = vec![
                http::header::ACCEPT,
                http::header::ACCEPT_CHARSET,
                http::header::ACCESS_CONTROL_ALLOW_ORIGIN,
                http::header::ACCESS_CONTROL_ALLOW_HEADERS,
                http::header::ACCESS_CONTROL_ALLOW_CREDENTIALS,
                http::header::ACCESS_CONTROL_ALLOW_METHODS,
                http::header::ACCESS_CONTROL_MAX_AGE,
                http::header::CONTENT_TYPE,
        ];
        let methods = ["GET", "POST", "OPTIONS"];

        Cors::default()
            .allowed_methods(methods)
            .allowed_headers(headers)
            .allowed_origin_fn(HeaderSettings::filter_origin)
            .max_age(3600)
    }
}