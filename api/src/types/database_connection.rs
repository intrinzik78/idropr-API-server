use std::{fs::File, io::Read};

use sqlx::mysql::{MySqlSslMode,MySqlConnectOptions,MySqlPool};

use crate::{
    enums::{
        ConnectionStatus,
         Error
    },
    types::Env
};

type Result<T> = std::result::Result<T,Error>;

#[derive(Clone,Debug)]
pub struct DatabaseConnection {
    pub pool: MySqlPool
}

impl DatabaseConnection {
    /// generates pem_cert for ssl server connection
    async fn pem_cert(path: &str) -> Result<Vec<u8>> {
        // buffer
        let mut pem_certificate: Vec<u8> = Vec::with_capacity(5192);

        // file handle
        let mut file_handle = File::open(path)
            .map_err(|e| Error::StdError(e.to_string()))?;
        
        // read into buffer
        let bytes_read = &mut file_handle
            .read_to_end(&mut pem_certificate)
            .map_err(|e| Error::StdError(e.to_string()))?;

        // success check
        if *bytes_read != pem_certificate.len() {
            return Err(Error::PemCertFileReadSizeMismatch);
        }

        Ok(pem_certificate)
    }

    /// builder function
    pub async fn new(env: &Env) -> Result<DatabaseConnection> {
        // load env variables
        let db_user = env.db_user();
        let db_port = env.db_port();
        let db_database = env.db_database();
        let db_host = env.db_host();
        let db_password = env.db_password();
        let db_cert_path = env.db_cert_path();
        let pem_certificate = DatabaseConnection::pem_cert(db_cert_path).await?;

        // connection options
        let options: MySqlConnectOptions = MySqlConnectOptions::new()
            .ssl_ca_from_pem(pem_certificate)
            .ssl_mode(MySqlSslMode::Required)
            .host(db_host)
            .username(db_user)
            .port(db_port)
            .database(db_database)
            .password(db_password);

        let pool = MySqlPool::connect_with(options)
            .await
            .map_err(|e| Error::DatabaseConnection(e.to_string()))?;

        let database_conection = DatabaseConnection {
            pool
        };

        Ok(database_conection)
    }

    /// checks connection state
    pub async fn connection_status(&self) -> ConnectionStatus {
        // test for active connections
        let current_status = match &self.pool.size() {
            0 => ConnectionStatus::Disconnected,
            _ => ConnectionStatus::Connected
        };
        
        // aquire a connection if currently disconnected
        if current_status == ConnectionStatus::Disconnected {
            match &self.pool.acquire().await {
                Ok(_) => ConnectionStatus::Connected,
                Err(_) => ConnectionStatus::Disconnected
            }
        } else {
            ConnectionStatus::Connected
        }
    }
}


#[cfg(test)]
pub mod test {
    use super::*;

    /// tests database connection and pool connection
    #[actix_rt::test]
    async fn connection_status() {
        let env = Env::default();

        let database = DatabaseConnection::new(&env).await.expect("failed to connect to database");
        let connection_status = database.connection_status().await;

        assert_eq!(connection_status, ConnectionStatus::Connected);
    }
}