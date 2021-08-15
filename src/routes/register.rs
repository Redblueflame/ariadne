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
use actix_web::dev::ConnectionInfo;
use std::cell::Ref;
use crate::geoip::database::GeoIp;
use std::sync::Arc;
use crate::models::download::Download;
use crate::error::AriadneErrors;
use log::kv::Source;
use reqwest::header::Keys;

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
    let val = match get_ip(conn_info) {
        None => {
            return HttpResponse::NotAcceptable().json(Error {
                name: "invalid_ip",
                description: "The ip is not parsable or is simply invalid."
            })
        }
        Some(e) => e
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

    let (lat, long, country_code) = get_city(val, ctx.geoip.clone());


    let visit = Visit {
        visit_id: Uuid::new_v4(),
        time: Utc::now().with_timezone(&Tz::GMT),
        user_agent,
        consent: info.consent.clone(),
        domain: info.domain.clone(),
        path: info.path.clone(),
        ip: val,
        country_code,
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

//noinspection ALL
#[post("/download")]
pub async fn register_download(req: HttpRequest, info: Json<DownloadInfo>, ctx: web::Data<AppState>) -> impl Responder {
    // This call is authenticated, so we need to check for the auth token.
    // TODO: Make this use minos' server2server token
    let token = req.headers().get(HeaderName::from_static("authorization"));
    if token.is_none() {
        return HttpResponse::Unauthorized().json(Error {
            name: "token_missing",
            description: "This endpoint needs a token."
        })
    }
    let val: String = token.unwrap().to_str().unwrap().to_string();
    if !val.starts_with("Bearer ") {
        return HttpResponse::NotAcceptable().json(Error {
            name: "invalid_token_type",
            description: "This endpoint only accepts a `Bearer` token."
        })
    }
    let token = &val[7..];
    println!("Token: \"{}\"", token);
    if token != ctx.config.read().unwrap().auth.server_token {
        return HttpResponse::NotAcceptable().json(Error {
            name: "invalid_token",
            description: "The token sent with this request is invalid."
        })
    }
    let ip = match Ipv4Addr::from_str(&*info.ip) {
        Ok(e) => e.to_ipv6_compatible(),
        Err(_) => match Ipv6Addr::from_str(&*info.ip) {
            Ok(e) => e,
            Err(_) => {
                return HttpResponse::NotAcceptable().json(Error {
                    name: "invalid_ip",
                    description: "The ip is not parsable or is simply invalid."
                })
            }
        }
    };
    let (lat, long, country_code) = get_city(ip, ctx.geoip.clone());

    // Build the download
    let download = Download {
        download_id: Uuid::new_v4(),
        time: Utc::now().with_timezone(&Tz::GMT),
        mod_id: info.mod_id.clone(),
        version_id: info.version_id.clone(),
        file_name: info.file_name.clone(),
        ip,
        user_agent: info.user_agent.clone(),
        country_code,
        latitude: lat,
        longitude: long
    };

    match ctx.database.insert_download(download).await {
        Ok(_) => {},
        Err(e) => {
            error!("There was an error while inserting the visit! {:#?}", e);
            return HttpResponse::InternalServerError().body("");
        }
    }
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

fn get_ip(conn: Ref<ConnectionInfo>) -> Option<Ipv6Addr> {
    let ip = match conn.realip_remote_addr().clone() {
        Some(e) => e.split(":").next().unwrap(),
        None => {
            return None
        }
    };
    Some(match Ipv4Addr::from_str(ip) {
        Ok(e) => e.to_ipv6_compatible(),
        Err(_) => match Ipv6Addr::from_str(ip) {
            Ok(e) => e,
            Err(_) => {
                return None
            }
        }
    })
}

fn get_city(ip: Ipv6Addr, geoip: Arc<GeoIp>) -> (Option<f64>, Option<f64>, Option<String>) {
    // Get geoip info:
    let city = match geoip.lookup(ip) {
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
    let country_code = get_country_code(&city);
    (lat, long, country_code)
}