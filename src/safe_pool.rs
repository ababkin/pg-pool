use anyhow::{Result, Error};
use deadpool_postgres::{Client, Manager, Config as PoolConfig, Pool, ManagerConfig, RecyclingMethod};
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use url::Url;
use tokio::sync::Mutex;
use tracing::{error, debug, warn, info};


#[derive(Debug)]
pub struct SafePool{
    pool: Pool
}

impl SafePool {
    pub fn new(url: String) -> Result<Self, Error> { 
        create(&url).map(|pool| Self { pool })
    }

    // pub async fn invalidate(&self) -> () {
    //     let mut lock = self.pool.lock().await;
    //     *lock = None;
    // }

    // pub async fn ensure(&self) {
    //     loop {
    //         debug!("Trying to open Postgres at {} ...", self.url);
    //         match create(&self.url) {
    //             Ok(pool) => {
    //                 let mut locked = self.pool.lock().await;
    //                 *locked = Some(pool);
    //                 debug!("Connected to Postgres.");
    //                 break;
    //             },
    //             Err(e) => {
    //                 error!("Failed to connect to Postgres: {:?}", e);
    //                 tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Wait before retrying
    //             }
    //         }
    //     }
    // }

    pub async fn get(&self) -> Result<Client, Error> {
        // loop {
        //     {
        //         let lock = self.pool.lock().await;
        //         if let Some(pool) = &*lock {
        //             return Ok(pool.get().await?);  // Clone the channel before returning
        //         }
        //     }
        //     warn!("PG connection lost, attempting to reconnect...");
        //     self.ensure().await;
        //     // tokio::time::sleep(tokio::time::Duration::from_secs(1)).await; // Wait before retrying
        // }
        Ok(self.pool.get().await?)
    }

}

fn config_from_url(url: &str) -> Result<PoolConfig, Error> {
    let url = Url::parse(url)?;
    let mut cfg = PoolConfig::new();

    cfg.user = Some(url.username().to_owned());
    cfg.password = url.password().map(|p| p.to_owned());
    cfg.host = Some(url.host_str().unwrap_or("").to_owned());
    cfg.port = url.port().or(Some(5432));  // Default to 5432 if no port is specified
    cfg.dbname = Some(url.path().trim_start_matches('/').to_owned());

    Ok(cfg)
}

// fn pg_config_from_url(url: &str) -> Result<tokio_postgres::Config> {
//     let url = Url::parse(url)?;

//     let mut pg_config = tokio_postgres::Config::new();
//     pg_config.host_path(url.host_str().unwrap_or(""));
//     pg_config.port(url.port().unwrap_or(5432));
//     pg_config.user(url.username());
//     pg_config.password(url.password().unwrap_or(""));
//     pg_config.dbname(url.path().trim_start_matches('/'));

//     Ok(pg_config)
// }

fn create(url: &str) -> Result<Pool, Error> {
    let pool_config = config_from_url(url)?;

    let mut builder = SslConnector::builder(SslMethod::tls())?;
    builder.set_verify(openssl::ssl::SslVerifyMode::NONE);  // Modify according to your security requirements
    let connector = MakeTlsConnector::new(builder.build());

    let pool = pool_config.create_pool(None, connector)?;
    pool.resize(16);


    // let pg_config = pg_config_from_url(&database_url)?;
    // let mgr_config = ManagerConfig {
    //     recycling_method: RecyclingMethod::Fast,
    // };
    // let mgr = Manager::from_config(pg_config, connector, mgr_config);
    // let pool = Pool::builder(mgr).max_size(16).build().unwrap();

    Ok(pool)
}
