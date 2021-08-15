use crate::connector::{DataConnector};
use crate::error::AriadneErrors;
use crate::models::download::Download;
use crate::models::visit::Visit;
use crate::settings::Settings;
use clickhouse_rs::types::Block;
use clickhouse_rs::Pool;
use log::{error, info};
use std::alloc::handle_alloc_error;
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;

pub struct ClickhouseConnector {
    pool: Pool,
    pub initialized: AtomicBool,
}

impl ClickhouseConnector {
    pub fn new(config: &Settings) -> Self {
        return Self {
            pool: Pool::new(config.database.url.clone()),
            initialized: AtomicBool::from(false),
        };
    }
}

#[async_trait]
impl DataConnector for ClickhouseConnector {
    async fn insert_visit(&self, _: Visit) -> Result<(), AriadneErrors> {
        unimplemented!()
    }

    async fn insert_download(&self, _: Download) -> Result<(), AriadneErrors> {
        unimplemented!()
    }

    async fn insert_mass_visits(&self, visits: &Vec<Visit>) -> Result<(), AriadneErrors> {
        let mut block = Block::new();
        // Columnize everything
        for visit in visits {
            block.push(visit.clone())?;
        }
        // Push it to the database:
        let mut handle = self.pool.get_handle().await?;
        handle.insert("visits", block).await?;
        Ok(())
    }

    async fn insert_mass_downloads(&self, downloads: &Vec<Download>) -> Result<(), AriadneErrors> {
        let mut block = Block::new();
        // Columnize everything
        for download in downloads {
            block.push(download.clone())?;
        }
        // Push it to the database:

        let mut handle = self.pool.get_handle().await?;
        handle.insert("downloads", block).await?;
        Ok(())
    }
}

pub async fn initialize_database(db: &ClickhouseConnector) -> Result<(), clickhouse_rs::errors::Error> {
    let creating_table_1 = r"
CREATE TABLE IF NOT EXISTS visits
(
    visit_id     UUID DEFAULT generateUUIDv4(),
    time         DateTime,
    path         String,
    domain       String,
    ip           IPv6,
    country_code  Nullable(FixedString(2)),
    latitude     Nullable(Float64),
    longitude    Nullable(Float64),
    user_agent   String,
    session_id   Nullable(String),
    consent      UInt8,
    time_on_page Nullable(UInt64)
) ENGINE = MergeTree() PRIMARY KEY visit_id";
    let creating_table_2 = r"
CREATE TABLE IF NOT EXISTS downloads
(
    download_id UUID DEFAULT generateUUIDv4(),
    time        DateTime,
    mod_id      String,
    version_id  String,
    file_name   String,
    ip          IPv6,
    user_agent  String,
    country_code Nullable(FixedString(2)),
    latitude    Nullable(Float64),
    longitude   Nullable(Float64)
) ENGINE = MergeTree() PRIMARY KEY download_id
    ";
    info!("Creating tables if not created...");
    let mut client = db.pool.get_handle().await?;
    client.execute(creating_table_1).await?;
    client.execute(creating_table_2).await?;
    Ok(())
}