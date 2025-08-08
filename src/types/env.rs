use std::collections::HashMap;

use dotenv;

use crate::{
    enums::ServerMode,
    traits::ToServerMode
};

// manages importing and testing of the .env file
pub struct Env {
    db_cert_path: String,       // path to local cert file
    db_user: String,            // username
    db_port: u16,               // default port is 3306
    db_database: String,        // schema / database name
    db_password: String,        // database access password
    db_host: String,            // ip address to host
    ip_address: String,         // server ip address
    master_password: String,    // for decrypting secret values on the database
    server_mode: ServerMode,    // [DEVELOPMENT,PRODUCTION,MAINTENANCE]
    server_port: u16            // port server will accept requests on
}

impl Env {
    pub fn db_user(&self) -> &str {
        &self.db_user
    }

    pub fn db_cert_path(&self) -> &str {
        &self.db_cert_path
    }

    pub fn db_port(&self) -> u16 {
        self.db_port
    }

    pub fn db_database(&self) -> &str {
        &self.db_database
    }

    pub fn db_password(&self) -> &str {
        &self.db_password
    }

    pub fn db_host(&self) -> &str {
        &self.db_host
    }

    pub fn ip_address(&self) -> &str {
        &self.ip_address
    }

    pub fn master_password(&self) -> &str {
        self.master_password.as_ref()
    }

    pub fn server_mode(&self) -> ServerMode {
        self.server_mode
    }

    pub fn server_port(&self) -> u16 {
        self.server_port
    }

}

impl Default for Env {
    fn default() -> Self {
        // load values
        let env:HashMap<String,String> = dotenv::vars().collect();

        // extracts env vars
        let db_cert_path = env.get("DB_CERT_PATH")
            .expect("DB_CERT_PATH not found in .env")
            .to_owned();
        let db_user = env.get("DB_USER")
            .expect("DB_USER not found in .env")
            .to_owned();
        let db_port: u16 = env.get("DB_PORT")
            .expect("DB_PORT not found in .env")
            .to_owned()
            .parse::<u16>().expect("could not parse DB_PORT field in .env")
            .to_owned();
        let db_database = env.get("DB_DATABASE")
            .expect("DB_DATABASE not found in .env")
            .to_owned();
        let db_password = env.get("DB_PASSWORD")
            .expect("DB_PASSWORD not found in .env")
            .to_owned();
        let db_host = env.get("DB_HOST")
            .expect("DB_HOST not found in .env")
            .to_owned();
        let ip_address = env.get("IP_ADDRESS")
            .expect("IP_ADDRESS not found in .env")
            .to_owned();
        let master_password = env.get("MASTER_PASSWORD")
            .expect("MASTER_PASSWORD not found in .env")
            .to_owned();
        let server_mode = env.get("SERVER_MODE")
            .expect("SERVER_MODE not found in .env")
            .to_owned()
            .to_server_mode()
            .expect("SERVER_MODE in .env out-of-range");
        let server_port = env.get("SERVER_PORT")
            .expect("SERVER_PORT not found in .env")
            .to_owned()
            .parse()
            .expect("could not parse SERVER_PORT in .env");

        Env {
            db_cert_path,
            db_user,
            db_port,
            db_database,
            db_password,
            db_host,
            ip_address,
            master_password,
            server_mode,
            server_port
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn default_env_builder() {
        // manually construct Env, will fail on missing values
        let manual_env = Env {
            db_cert_path: String::from("db_cert_path"),
            db_user: String::from("db_user"),
            db_port: String::from("3306").parse().unwrap(),
            db_database: String::from("db_database"),
            db_password: String::from("db_password"),
            db_host: String::from("db_host"),
            ip_address: String::from("ip_address"),
            master_password: String::from("master_password"),
            server_port: String::from("3000").parse().unwrap(),
            server_mode: ServerMode::Production
        };

        // test function calls return correct data
        assert_eq!(manual_env.db_cert_path(), String::from("db_cert_path"));
        assert_eq!(manual_env.db_user(), String::from("db_user"));
        assert_eq!(manual_env.db_port(), 3306);
        assert_eq!(manual_env.db_database(), String::from("db_database"));
        assert_eq!(manual_env.db_password(), String::from("db_password"));
        assert_eq!(manual_env.db_host(), String::from("db_host"));
        assert_eq!(manual_env.ip_address(), String::from("ip_address"));
        assert_eq!(manual_env.master_password(), &String::from("master_password"));
        assert_eq!(manual_env.server_port(), 3000);
        assert_eq!(manual_env.server_mode(), ServerMode::Production);

        // test constructor generated properties contain some values
        let builder = Env::default();
        assert!(!builder.db_cert_path().is_empty());
        assert!(!builder.db_user().is_empty());
        assert!(!builder.db_database().is_empty());
        assert!(!builder.db_password().is_empty());
        assert!(!builder.db_host().is_empty());
        assert!(!builder.ip_address().is_empty());
        assert!(!builder.master_password().is_empty());
        assert!(builder.server_port() > 0);
        
    }
}