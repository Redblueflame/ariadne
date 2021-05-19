use crate::connector::clickhouse::ClickhouseConnector;
use crate::error::AriadneErrors;
use crate::models::download::Download;
use crate::models::visit::Visit;
use crate::settings::{DatabaseType, Settings};
use async_trait::async_trait;
use thiserror::Error;

pub mod cache_data;
pub mod clickhouse;

pub fn test_send<S>(ty: &S) where S: Send {

}

#[async_trait]
pub trait DataConnector {
    async fn insert_visit(&self, visit: Visit) -> Result<(), AriadneErrors>;
    async fn insert_download(&self, download: Download) -> Result<(), AriadneErrors>;
    async fn insert_mass_visits(&self, visits: &Vec<Visit>) -> Result<(), AriadneErrors>;
    async fn insert_mass_downloads(&self, downloads: &Vec<Download>) -> Result<(), AriadneErrors>;
}

pub fn get_implementation(config: &Settings) -> ClickhouseConnector {
    return match config.database.db_type {
        DatabaseType::ClickHouse => ClickhouseConnector::new(config),
        DatabaseType::TensorBase => unimplemented!(),
    };
}
