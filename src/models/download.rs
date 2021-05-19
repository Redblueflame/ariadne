use chrono::{DateTime, Utc};
use clickhouse_rs::types::{ColumnType, RowBuilder, Value};
use clickhouse_rs::Block;
use clickhouse_rs::row;
use std::net::Ipv6Addr;
use chrono_tz::Tz;
use uuid::Uuid;
use std::collections::HashMap;

/// Represents a download in the analytics database.
/// It holds data strictly necessary for threat protection,
/// This data will only appear grouped on the front facing APIs, and only for the fields indicated in this document.
#[derive(Debug, Clone)]
pub struct Download {
    /// A snowflake generated ID.
    download_id: Uuid,
    /// The time the download was made
    /// It is in UTC for consistency, and is generated automatically.
    time: DateTime<Tz>,
    /// The ModID of the requested download
    mod_id: String,
    /// The version id of the requested download
    version_id: String,
    /// The file name of the requested download
    /// This is made so statistics can be brought up of what file of a specific version was downloaded by users
    file_name: String,
    /// The ip of the user that made the request (if enabled in the settings)
    /// NOTE: This PII is simply not used for anything else than threat prevention,
    /// it is used to protect us from botting to get as much downloads as possible.
    ip: Ipv6Addr,
    /// The user agent used to make the request.
    /// It is logged by the analytics server itself, to avoid spoofing
    user_agent: String,
    /// The country code of the client that made the request (if available)
    /// The data is generated from the maxmind database
    country_code: Option<String>,
    /// The latitude of the client that made the request (if available)
    /// The data is generated from the maxmind database
    latitude: Option<f64>,
    /// The longitude of the request (if available)
    /// The data is generated from the maxmind database
    longitude: Option<f64>,
    /// Checks if the user has a proxy or VPN enabled.
    /// This field is not 100% accurate, as it's provided from maxminds database.
    is_proxy: bool,
}
impl RowBuilder for Download {
    fn apply<K: ColumnType>(
        self,
        block: &mut Block<K>,
    ) -> Result<(), clickhouse_rs::errors::Error> {
        block.push(row!{
            download_id: Value::Uuid(*self.download_id.as_bytes()),
            time: self.time,
            mod_id: self.mod_id,
            version_id: self.version_id,
            file_name: self.file_name,
            ip: self.ip.to_string(),
            user_agent: self.user_agent,
            country_code: self.country_code,
            latitude: self.latitude,
            longitude: self.longitude,
            is_proxy: if self.is_proxy { 1_u8 } else { 0_u8 }
        })?;
        Ok(())
    }
}
