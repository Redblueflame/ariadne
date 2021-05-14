use actix_web::web;
use crate::routes::register::import_routes;
use serde::Serialize;
mod register;

pub fn routes_import(cfg: &mut web::ServiceConfig) {
    // Add services here:
    cfg.service(import_routes());
}
#[derive(Serialize)]
pub struct Error<'s> {
    name: &'s str,
    description: &'s str
}