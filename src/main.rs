use actix_web::{web, App, HttpServer, http};
use crate::routes::routes_import;
use std::sync::{Arc, Mutex, RwLock};
use crate::connector::cache_data::CacheData;
use crate::scheduler::Scheduler;
use crate::settings::Settings;
use crate::geoip::database::GeoIp;
use log::{warn, LevelFilter};
use std::str::FromStr;
use crate::connector::clickhouse::initialize_database;
use actix_cors::Cors;

mod connector;
mod error;
mod models;
mod scheduler;
mod settings;
mod routes;
mod geoip;

// This struct represents state
pub struct AppState {
    database: Arc<CacheData>,
    config: Arc<RwLock<Settings>>,
    geoip: Arc<GeoIp>
}

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let config = Settings::new().unwrap();

    env_logger::builder()
        .filter_level(LevelFilter::from_str(&*config.log.level).unwrap())
        .init();
    let mut scheduler = scheduler::Scheduler::new();
    let database = get_database(&mut scheduler, &config);
    let geoip = Arc::new(GeoIp::from_config(&config)?);
    let config_arc = Arc::new(RwLock::new(config.clone()));
    // Load config from env
    Ok(HttpServer::new(move || {

        let cors = Cors::default()
            .allowed_origin_fn(|origin, _req_head| {
                let val = origin.to_str();
                println!("host: {:#?}", val);
                let val = val.unwrap_or("");
                val.contains("127.0.0.1") || val.contains("localhost") || val.contains("modrinth.com")
            })
            .allowed_methods(vec!["GET", "POST"])
            .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
            .allowed_header(http::header::CONTENT_TYPE)
            .max_age(3600);
        App::new()
            .wrap(cors)
            .data(AppState {
                database: database.clone(),
                config: config_arc.clone(),
                geoip: geoip.clone()
            })
            .configure(routes_import)
    })
    .bind((config.server.host, config.server.port))?
    .run()
    .await?)
}
fn get_database(scheduler: &mut Scheduler, config: &Settings) -> Arc<CacheData> {
    if !config.database.batching {
        warn!("The batch saving scheduler is not enabled, note that this is not supported yet.");
    }
    let db = Arc::new(CacheData::new(config));

    scheduler::register_saving_scheduler(scheduler, db.clone(), config);
    return db
}