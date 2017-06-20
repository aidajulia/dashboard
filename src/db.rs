//! includes database, models (like Dashboard), etc.

// TODO: migrate to postgres?, and rm it

use redis;
use redis::Commands;
use serde_json;
use utils::{get_redis_con, from_config};
use uuid::Uuid;


pub struct Db {
    connection: redis::Connection,
}

const DASHBOARDS_KEY: &'static str = "dashboards";

impl Db {
    pub fn new() -> Result<Db, &'static str> {
        let connection = get_redis_con(from_config("DASHBOARD_REDIS_IP_PORT").as_str())?;
        let db = Db { connection: connection };
        Ok(db)
    }

    /// Saves `Dashboard` at `Dashboard.name` in redis
    ///
    /// # Errors
    /// Raises error when `Dashboard.name` already exists
    pub fn create_dashboard(&self, dashboard: &Dashboard) -> Result<(), String> {
        match self.connection
            .hexists::<_, _, bool>(DASHBOARDS_KEY, &dashboard.name) {
            Err(e) => return Err(e.to_string()),
            Ok(true) => return Err(format!("Dashboard {} already exists", dashboard.name)),
            Ok(false) => (),
        }
        // TODO: impl to ToRedisArgs so this line could be removed?
        let json = serde_json::to_string(&dashboard)
            .map_err(|e| e.to_string())?;
        self.connection
            .hset::<_, _, _, u64>(DASHBOARDS_KEY, &dashboard.name, &json)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Returns `Dashboard` from saved at `dashboard_name`
    pub fn get_dashboard(&self, dashboard_name: &str) -> Result<Option<String>, String> {
        self.connection
            .hget::<_, _, Option<String>>(DASHBOARDS_KEY, dashboard_name)
            .map_err(|e| e.to_string())
    }

    /// Deletes `Dashboard` at `dashboard_name`
    pub fn delete_dashboard(&self, dashboard_name: &str) -> Result<(u64), String> {
        self.connection
            .hdel::<_, _, u64>(DASHBOARDS_KEY, dashboard_name)
            .map(|v| v)
            .map_err(|e| e.to_string())
    }
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dashboard {
    pub name: String,
    pub owner_email: String,
    // TODO: enum?
    pub layout: String,
    api_token: Option<String>,
}

impl Dashboard {
    /// Generates and assigns api token
    pub fn assign_api_token(&mut self) {
        self.api_token = Some(Uuid::new_v4().to_string());
    }

    /// Returns api token
    pub fn get_api_token(&self) -> Option<&String> {
        self.api_token.as_ref()
    }
}
