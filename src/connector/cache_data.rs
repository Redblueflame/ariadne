use crate::connector::clickhouse::{ClickhouseConnector, initialize_database};
use crate::connector::{get_implementation, DataConnector};
use crate::error::AriadneErrors;
use crate::models::download::Download;
use crate::models::visit::Visit;
use crate::settings::Settings;
use async_trait::async_trait;
use crossbeam::queue::SegQueue;
use log::info;
use std::{collections::VecDeque, sync::Arc};
use std::sync::atomic::Ordering;

pub struct CacheData {
    implementation: ClickhouseConnector,
    pending_visits: SegQueue<Visit>,
    pending_downloads: SegQueue<Download>,
}
#[async_trait]
impl DataConnector for CacheData {
    async fn insert_visit(&self, visit: Visit) -> Result<(), AriadneErrors> {
        self.pending_visits.push(visit);
        Ok(())
    }

    async fn insert_download(&self, download: Download) -> Result<(), AriadneErrors> {
        self.pending_downloads.push(download);
        Ok(())
    }

    async fn insert_mass_visits(&self, visits: Vec<Visit>) -> Result<(), AriadneErrors> {
        unimplemented!()
    }

    async fn insert_mass_downloads(&self, downloads: Vec<Download>) -> Result<(), AriadneErrors> {
        unimplemented!()
    }
}
impl CacheData {
    pub fn new(config: &Settings) -> Self {
        Self {
            implementation: get_implementation(config),
            pending_visits: SegQueue::new(),
            pending_downloads: SegQueue::new(),
        }
    }
}

pub async fn sync(db: Arc<CacheData>) -> Result<(), AriadneErrors> {
    if !db.implementation.initialized.load(Ordering::Relaxed) {
        initialize_database(&db.implementation).await?;
        db.implementation.initialized.store(true, Ordering::Relaxed);
    }
    let mut vec: Vec<Visit> = vec![];
    while let Some(visit) = db.pending_visits.pop() {
        vec.push(visit);
    }
    if vec.len() > 0 {
        info!("Synchronizing {} visits.", vec.len());
        db.implementation.insert_mass_visits(vec).await?;
    }
    Ok(())
}
