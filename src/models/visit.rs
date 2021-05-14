use chrono::{DateTime, Utc};
use clickhouse_rs::types::{ColumnType, RowBuilder, Value, SqlType};
use clickhouse_rs::Block;
use chrono::TimeZone;
use chrono_tz::Tz;
use std::net::Ipv6Addr;
use uuid::Uuid;
use nom::AsBytes;
use clickhouse_rs::row;
use crate::models::{Serializable, AriadneBlock};
use serde_json::map::Entry::Vacant;

/// Represents a user in the analytics database.
/// It holds the bare minimum of data required for threat prevention.
#[derive(Debug, Clone)]
pub struct Visit {
    /// A snowflake generated ID.
    pub visit_id: Option<Uuid>,
    /// The time the request were made
    /// It is in UTC for consistency, and is generated automatically.
    pub time: DateTime<Tz>,
    /// The relative url of the request
    /// For example, if the request is `https://modrinth.com/test/mod`,
    /// this field will contain `/test/mod`
    pub path: String,
    /// The base domain of the request
    /// For example, if the request is `https://modrinth.com/test/mod`,
    /// this field will contain `modrinth.com`
    pub domain: String,
    /// The user agent used to make the request.
    /// It is logged by the analytics server itself, to avoid spoofing
    pub user_agent: String,
    /// The ip of the user that made the request (if enabled in the settings)
    pub ip: Ipv6Addr,
    /// The country code of the client that made the request (if available)
    /// The data is generated from the maxmind database
    pub country_code: Option<String>,
    /// The latitude of the client that made the request (if available)
    /// The data is generated from the maxmind database
    pub latitude: Option<f64>,
    /// The longitude of the request (if available)
    /// The data is generated from the maxmind database
    pub longitude: Option<f64>,
    /// The session id.
    /// This is only available if the user has accepted the enhanced analytics scope in the privacy page
    /// And simply set to [None](std::option::Option::None) if it was not enabled.
    pub session_id: Option<String>,
    /// Consent check, is set to true if the user has enabled the enhanced analytics scope in the privacy page
    /// Is set to true if there was a consent.
    /// NOTE: Every row where the consent was not given will still appear in the modders / administrative dashboard
    /// but all PII (IP and lat + long) will be striped before being stocked in another database in our analytics system.
    /// (This is currently not implemented)
    pub consent: bool,
    /// Estimate of the amount of time spent on a page.
    /// This is sent by the client if the user switches between pages,
    /// or if the page has enough time to make a AJAX request to this server.
    /// This is only available if the user has accepted the enhanced analytics scope in the privacy page
    /// And simply set to [None](std::option::Option::None) if it was not enabled.
    pub time_on_page: Option<u64>,
}
impl Serializable for Visit {
    fn export(self, block: &mut AriadneBlock) {

    }
}
impl RowBuilder for Visit {
    fn apply<K: ColumnType>(
        self,
        block: &mut Block<K>,
    ) -> Result<(), clickhouse_rs::errors::Error> {
        block.push(row!{
            //visit_id: self.visit_id,
            time: self.time,
            path: self.path,
            domain: self.domain,
            ip: self.ip.to_string(),
            user_agent: self.user_agent,
            country_code: self.country_code,
            latitude: self.latitude,
            longitude: self.longitude,
            session_id: self.session_id,
            consent: if self.consent { 1_u8 } else { 0_u8 },
            time_on_page: self.time_on_page
        })?;
        println!("{:#?}", block);
        Ok(())
    }
}
