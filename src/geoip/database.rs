use maxminddb::{Reader, MaxMindDBError};
use std::net::{Ipv6Addr, IpAddr};
use maxminddb::geoip2::City;
use crate::settings::Settings;
use anyhow::anyhow;
use std::path::Path;
use std::fs;

pub struct GeoIp {
    // Note: the database (~100Mb) is loaded in memory for higher performance.
    inner: Reader<Vec<u8>>
}
impl GeoIp {
    pub fn new(path: &Path) -> Result<Self, MaxMindDBError> {
        Ok(GeoIp {
            inner: Reader::open_readfile(path)?
        })
    }
    pub fn from_config(data: &Settings) -> anyhow::Result<Self> {
        if let Some(maxmind) = data.maxmind.clone() {
            if let Some(path) = maxmind.path.clone() {
                let path = Path::new(&path);
                return Ok(Self::new(path)?)
            }
        }
        return Err(anyhow!("There is no configuration for the maxmind database."))

    }
    pub fn lookup(&self, ip: Ipv6Addr) -> Result<City, MaxMindDBError> {
        let v4 = ip.to_ipv4();
        if v4.is_some() {
            // Use the Ipv4
            let val: City = self.inner.lookup(IpAddr::from(v4.unwrap()))?;
            Ok(val)
        } else {
            // Use Ipv6 directly
            let val: City = self.inner.lookup(IpAddr::from(ip))?;
            Ok(val)
        }
    }
}