use actix_web::{post, Responder, HttpResponse, web, Scope, HttpRequest};
use serde::Deserialize;
use actix_web::web::Json;
use actix_web::http::header::USER_AGENT;
use log::info;
use crate::models::visit::Visit;
use crate::routes::Error;
use uuid::Uuid;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use crate::AppState;
use crate::connector::DataConnector;

#[derive(Deserialize)]
pub struct VisitInfo {
    path: String,
    domain: String,
    consent: bool
}

#[post("/visit")]
pub async fn register_visit(req: HttpRequest, info: Json<VisitInfo>, ctx: web::Data<AppState>) -> impl Responder {
    let user_agent = match req.headers().get(USER_AGENT) {
        Some(e) => e,
        None => {
            return HttpResponse::NotAcceptable().json(Error {
                name: "invalid_user_agent",
                description: "The UserAgent field is required."
            })
        }
    };
    let conn_info = req.connection_info();
    let ip = match conn_info.remote_addr().clone() {
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
    let visit = Visit {
        visit_id: None,
        time: Utc::now().with_timezone(&Tz::GMT),
        user_agent: user_agent.to_str().unwrap().to_string(),
        consent: info.consent.clone(),
        domain: info.domain.clone(),
        path: info.path.clone(),
        ip: val,
        country_code: None,
        latitude: None,
        longitude: None,
        session_id: None,
        time_on_page: None
    };
    ctx.database.insert_visit(visit).await.unwrap();
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