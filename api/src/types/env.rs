use std::{collections::HashMap, fmt::Debug};

use dotenv;
use rate_limit::{enums::TimeWindow,traits::ToTimeWindow};
use crate::{
    enums::ServerMode,
    traits::ToServerMode
};

// manages importing and testing of the .env file
#[derive(Debug)]
pub struct Env {
    // database settings
    pub db_cert_path: String,       // path to local cert file
    pub db_user: String,            // username
    pub db_port: u16,               // default port is 3306
    pub db_database: String,        // schema / database name
    pub db_password: String,        // database access password
    pub db_host: String,            // ip address to host

    // api server settings
    pub ip_address: String,         // server ip address
    pub master_password: String,    // for decrypting secret values on the database
    pub server_mode: ServerMode,    // [DEVELOPMENT,PRODUCTION,MAINTENANCE]
    pub server_port: u16,           // port server will accept requests on
    pub server_threads: usize,      // maximum number of thread workers

    // rate limiter settings
    pub limiter_initial_capacity: usize,
    pub limiter_tokens_per_bucket: u32,
    pub limiter_initial_tokens_per_bucket: u32,
    pub limiter_refill_rate: f32,
    pub limiter_refill_window: TimeWindow,

    // session controller settings
    pub sessions_initial_capacity: usize
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
            .parse()
            .expect("could not parse DB_PORT field in .env");
 
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

        let limiter_initial_capacity = env.get("LIMITER_INITIAL_CAPACITY")
            .expect("LIMITER_INITIAL_CAPACITY not found in .env")
            .to_owned()
            .parse()
            .expect("could not parse LIMITER_INITIAL_CAPACITY in .env");

        let limiter_tokens_per_bucket = env.get("LIMITER_TOKENS_PER_BUCKET")
            .expect("LIMITER_TOKENS_PER_BUCKET not found in .env")
            .to_owned()
            .parse()
            .expect("could not parse LIMITER_TOKENS_PER_BUCKET in .env");

        let server_threads: usize = env.get("SERVER_THREADS")
            .expect("SERVER_THREADS not found in .env")
            .parse()
            .expect("could not parse SERVER_THREADS in .env");

        let limiter_refill_rate: f32 = env.get("LIMITER_REFILL_RATE")
            .expect("LIMITER_REFILL_RATE not found in .env")
            .parse()
            .expect("could not parse LIMITER_REFILL_RATE in .env");

        let limiter_refill_window: TimeWindow = env.get("LIMITER_REFILL_WINDOW")
            .expect("LIMITER_REFILL_WINDOW not found in .env")
            .to_time_window()
            .expect("LIMITER_REFILL_WINDOW out of bounds in .env");

        let limiter_initial_tokens_per_bucket: u32 = env.get("LIMITER_INITIAL_TOKENS_PER_BUCKET")
            .expect("LIMITER_INITIAL_TOKENS_PER_BUCKET not found in .env")
            .parse()
            .expect("could not parse LIMITER_INITIAL_TOKENS_PER_BUCKET in .env");

        let sessions_initial_capacity: usize = env.get("SESSIONS_INITIAL_CAPACITY")
            .expect("SESSIONS_INITIAL_CAPACITY not found in .env")
            .parse()
            .expect("could not parse SESSIONS_INITIAL_CAPACITY in .env");

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
            server_port,
            limiter_initial_capacity,
            limiter_initial_tokens_per_bucket,
            limiter_refill_rate,
            limiter_refill_window,
            limiter_tokens_per_bucket,
            server_threads,
            sessions_initial_capacity
        }
    }
}

#[cfg(test)]
mod tests {
    use rate_limit::traits::ToTimeWindow;

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
            server_mode: ServerMode::Production,
            server_threads: 2,
            limiter_initial_capacity: String::from("100").parse().unwrap(),
            limiter_initial_tokens_per_bucket: 1000,
            limiter_tokens_per_bucket: String::from("100").parse().unwrap(),
            limiter_refill_rate: String::from("100").parse().unwrap(),
            limiter_refill_window: String::from("HOUR").to_time_window().unwrap(),
            sessions_initial_capacity: String::from("1000").parse().unwrap()
        };

        // test function calls return correct data
        assert_eq!(manual_env.db_cert_path, String::from("db_cert_path"));
        assert_eq!(manual_env.db_user, String::from("db_user"));
        assert_eq!(manual_env.db_port, 3306);
        assert_eq!(manual_env.db_database, String::from("db_database"));
        assert_eq!(manual_env.db_password, String::from("db_password"));
        assert_eq!(manual_env.db_host, String::from("db_host"));
        assert_eq!(manual_env.ip_address, String::from("ip_address"));
        assert_eq!(manual_env.master_password, String::from("master_password"));
        assert_eq!(manual_env.server_port, 3000);
        assert_eq!(manual_env.server_mode, ServerMode::Production);
        assert_eq!(manual_env.limiter_initial_capacity, 100);
        assert_eq!(manual_env.limiter_initial_tokens_per_bucket, 1000);
        assert_eq!(manual_env.limiter_tokens_per_bucket, 100);
        assert_eq!(manual_env.limiter_refill_window,TimeWindow::Hour);
        assert_eq!(manual_env.server_threads, 2);
        assert_eq!(manual_env.sessions_initial_capacity, 1000);

        // test constructor generated properties contain some values
        let builder = Env::default();
        assert!(!builder.db_cert_path.is_empty());
        assert!(!builder.db_user.is_empty());
        assert!(!builder.db_database.is_empty());
        assert!(!builder.db_password.is_empty());
        assert!(!builder.db_host.is_empty());
        assert!(!builder.ip_address.is_empty());
        assert!(!builder.master_password.is_empty());
        assert!(builder.server_port > 0);
        assert!(builder.limiter_initial_capacity > 0);
        assert!(builder.limiter_initial_tokens_per_bucket > 0);
        assert!(builder.limiter_tokens_per_bucket > 0);
        assert!(builder.server_threads > 0);
        assert!(builder.sessions_initial_capacity > 0);

        // exhaustive
        let time_window = match builder.limiter_refill_window {
            TimeWindow::Day => true,
            TimeWindow::Hour => true,
            TimeWindow::Minute => true,
            TimeWindow::Second => true
        };

        assert!(time_window);
        
    }
}