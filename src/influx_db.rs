use influxdb::{Client, Error, Query};
use std::env::var;

use anyhow::Result;
use serde::Deserialize;

#[derive(Deserialize, Debug, Clone)]
pub struct StorageUsage {
    pub time: String,
    pub hardware: String,
    pub value: f64,
}

pub struct InfluxDBConnection {
    pub client: Client,
}

impl InfluxDBConnection {
    pub fn new() -> Result<Self> {
        let hostname = var("DB_HOST")?;
        let database = var("DB_NAME")?;
        let username = var("DB_USER")?;
        let password = var("DB_PASSWORD")?;

        let client = Client::new(&hostname, &database).with_auth(&username, &password);

        Ok(Self { client })
    }

    pub async fn get_storage_load(&self) -> Result<Vec<StorageUsage>, Error> {
        let read_query = Query::raw_read_query("SELECT hardware, value FROM Load WHERE app = 'ohm' and hardware_type = 'HDD' and sensor = 'Used Space' GROUP BY hardware LIMIT 1");
        let mut result = self.client.json_query(read_query).await?;
        Ok(result
            .deserialize_next::<StorageUsage>()?
            .series
            .iter()
            .map(|series| series.values[0].clone())
            .collect())
    }
}
