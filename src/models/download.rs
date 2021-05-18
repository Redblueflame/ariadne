use chrono::{DateTime, Utc};
use clickhouse_rs::types::{ColumnType, RowBuilder, Value};
use clickhouse_rs::Block;
use std::net::Ipv6Addr;
use chrono_tz::Tz;
use uuid::Uuid;
use std::collections::HashMap;

/// Represents a download in the analytics database.
/// It holds data strictly necessary for threat protection,
/// This data will only appear grouped on the front facing APIs, and only for the fields indicated in this document.
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
        let val: Vec<(String, Value)> = vec![
            //("download_id".to_string(), Value::String(self.download_id.to_string())),
            ("time".to_string(), Value::from(self.time)),
            ("mod_id".to_string(), Value::from(self.mod_id)),
            ("version_id".to_string(), Value::from(self.version_id)),
            ("file_name".to_string(), Value::from(self.file_name)),
            ("ip".to_string(), Value::Ipv6(self.ip.octets())),
            ("user_agent".to_string(), Value::from(self.user_agent)),
            ("country_code".to_string(), Value::from(self.country_code)),
            ("latitude".to_string(), Value::from(self.latitude)),
            ("longitude".to_string(), Value::from(self.longitude)),
            ("is_proxy".to_string(), Value::from(if self.is_proxy { 1 } else { 0 })),
        ];
        block.push(val)
    }
}
