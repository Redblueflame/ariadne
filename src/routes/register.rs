use actix_web::{post, Responder, HttpResponse, web, Scope, HttpRequest};
use serde::Deserialize;
use actix_web::web::Json;
use actix_web::http::header::USER_AGENT;
use log::{info, error};
use crate::models::visit::Visit;
use crate::routes::Error;
use uuid::Uuid;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use crate::AppState;
use crate::connector::DataConnector;
use actix_web::http::{HeaderMap, HeaderName};
use maxminddb::geoip2::City;
use maxminddb::geoip2::model::Country;

#[derive(Deserialize)]
pub struct VisitInfo {
    path: String,
    domain: String,
    consent: bool,
    session_id: Option<String>
}

#[post("/visit")]
pub async fn register_visit(req: HttpRequest, info: Json<VisitInfo>, ctx: web::Data<AppState>) -> impl Responder {

    let conn_info = req.connection_info();
    let ip = match conn_info.realip_remote_addr().clone() {
        Some(e) => e.split(":").next().unwrap(),
        None => {
            return HttpResponse::NotAcceptable().json(Error {
                name: "invalid_remote_addr",
                description: "The remote address couldn't be extracted."
            })
        }
    };
    let val = match Ipv4Addr::from_str(ip) {
        Ok(e) => e.to_ipv6_compatible(),
        Err(_) => match Ipv6Addr::from_str(ip) {
            Ok(e) => e,
            Err(_) => {
                return HttpResponse::NotAcceptable().json(Error {
                    name: "invalid_remote_addr",
                    description: "The remote address couldn't be parsed to Ipv6."
                })
            }
        }
    };
    let user_agent = match get_user_agent(req.headers()) {
        Some(e) => e,
        None => {
            return HttpResponse::NotAcceptable().json(Error {
                name: "invalid_user_agent",
                description: "The user agent is invalid or not filled in."
            })
        }
    };

    // Get geoip info:
    let city = match ctx.geoip.lookup(val) {
        Ok(e) => e,
        Err(_) => {
            City {
                city: None,
                continent: None,
                country: None,
                location: None,
                postal: None,
                registered_country: None,
                represented_country: None,
                subdivisions: None,
                traits: None
            }
        }
    };
    let (lat, long) = get_latitude_longitude(&city);
    let visit = Visit {
        visit_id: Uuid::new_v4(),
        time: Utc::now().with_timezone(&Tz::GMT),
        user_agent,
        consent: info.consent.clone(),
        domain: info.domain.clone(),
        path: info.path.clone(),
        ip: val,
        country_code: get_country_code(&city),
        latitude: lat,
        longitude: long,
        session_id: None,
        time_on_page: None
    };
    match ctx.database.insert_visit(visit).await {
        Ok(_) => {},
        Err(e) => {
            error!("There was an error while inserting the visit! {:#?}", e);
            return HttpResponse::InternalServerError().body("");
        }
    };
    // Add the data
    HttpResponse::NoContent().body("")
}

#[derive(Deserialize)]
pub struct DownloadInfo {
    mod_id: String,
    version_id: String,
    file_name: String,
    ip: String,
    user_agent: String,
}

#[post("/download")]
pub async fn register_download(req: HttpRequest, info: Json<DownloadInfo>) -> impl Responder {
    HttpResponse::NoContent().body("")
}

pub fn import_routes() -> Scope {
    web::scope("/register")
        .service(register_download)
        .service(register_visit)
}

//noinspection ALL
fn get_user_agent(headers: &HeaderMap) -> Option<String> {
    // Check the fowarded user agent
    match headers.get(HeaderName::from_static("x-user-agent")) {
        Some(e) => match e.to_str() {
            Ok(e) => Some(e.to_string()),
            Err(_) => None
        },
        None => {
            // Use the main user agent
            match headers.get(USER_AGENT)?.to_str() {
                Ok(e) => Some(e.to_string()),
                Err(_) => None
            }
        }
    }
}

fn get_country_code(city: &City) -> Option<String> {
    Some(city.country.clone()?.iso_code.clone()?.to_string())
}
fn get_latitude_longitude(city: &City) -> (Option<f64>, Option<f64>) {
    let location = match city.location.clone() {
        Some(e) => e,
        None => return (None, None)
    };
    (location.latitude, location.longitude)
}
