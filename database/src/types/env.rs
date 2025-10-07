use std::{collections::HashMap, fmt::Debug, path::PathBuf};

use dotenv;

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
}

impl Env {
    fn workspace_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().to_path_buf()
    }
}

impl Default for Env {
    fn default() -> Self {
        let cwd = Self::workspace_root();

        // load values
        let env:HashMap<String,String> = dotenv::vars().collect();

        // extracts env vars
        let db_cert_path_rel = env.get("DB_CERT_PATH")
            .expect("DB_CERT_PATH not found in .env")
            .to_owned();

        let db_cert_path = cwd.join(db_cert_path_rel)
            .to_str()
            .expect("path to DB_CERT could not be created")
            .to_string();

        println!("{}",db_cert_path);

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

        Env {
            db_cert_path,
            db_user,
            db_port,
            db_database,
            db_password,
            db_host,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[actix_rt::test]
    async fn default_env_builder() {
        let _env = Env::default();

        // manually construct Env, will fail on missing values
        let manual_env = Env {
            db_cert_path: String::from("db_cert_path"),
            db_user: String::from("db_user"),
            db_port: String::from("3306").parse().unwrap(),
            db_database: String::from("db_database"),
            db_password: String::from("db_password"),
            db_host: String::from("db_host"),
        };

        // test function calls return correct data
        assert_eq!(manual_env.db_cert_path, String::from("db_cert_path"));
        assert_eq!(manual_env.db_user, String::from("db_user"));
        assert_eq!(manual_env.db_port, 3306);
        assert_eq!(manual_env.db_database, String::from("db_database"));
        assert_eq!(manual_env.db_password, String::from("db_password"));
        assert_eq!(manual_env.db_host, String::from("db_host"));

        // test constructor generated properties contain some values
        let builder = Env::default();
        assert!(!builder.db_cert_path.is_empty());
        assert!(!builder.db_user.is_empty());
        assert!(!builder.db_database.is_empty());
        assert!(!builder.db_password.is_empty());
        assert!(!builder.db_host.is_empty());        
    }
}