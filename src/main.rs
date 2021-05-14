use actix_web::{web, App, HttpServer};
use crate::routes::routes_import;
use std::sync::{Arc, Mutex, RwLock};
use crate::connector::cache_data::CacheData;
use crate::scheduler::Scheduler;
use crate::settings::Settings;
use log::{warn, LevelFilter};
use std::str::FromStr;
use crate::connector::clickhouse::initialize_database;

mod connector;
mod error;
mod models;
mod scheduler;
mod settings;
mod routes;

// This struct represents state
pub struct AppState {
    database: Arc<CacheData>,
    config: Arc<RwLock<Settings>>,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = Settings::new().unwrap();

    env_logger::builder()
        .filter_level(LevelFilter::from_str(&*config.log.level).unwrap())
        .init();
    let mut scheduler = scheduler::Scheduler::new();
    let database = get_database(&mut scheduler, &config);
    // Load config from env
    HttpServer::new(move || {
        let config = Arc::new(RwLock::new(Settings::new().unwrap()));
        App::new()
            .data(AppState {
                database: database.clone(),
                config: config.clone()
            })
            .configure(routes_import)
    })
    .bind((config.server.host, config.server.port))?
    .run()
    .await
}
fn get_database(scheduler: &mut Scheduler, config: &Settings) -> Arc<CacheData> {
    if !config.database.batching {
        warn!("The batch saving scheduler is not enabled, note that this is not supported yet.");
    }
    let db = Arc::new(CacheData::new(config));

    scheduler::register_saving_scheduler(scheduler, db.clone(), config);
    return db
}