//! includes database, models (like Dashboard), etc.

// TODO: migrate to postgres?, and rm it

use natord;
use redis;
use redis::Commands;
use serde_json;
use std::error::Error;
use utils::{get_redis_con, from_config};
use uuid::Uuid;


pub struct Db {
    connection: redis::Connection,
}

const DASHBOARDS_KEY: &'static str = "dashboards";
const TILES_KEY: &'static str = "tiles";


/// Returns channel for `dashboard_name` dashboard where changes are announced
pub fn get_dashboard_channel<D: AsRef<str>>(dashboard_name: D) -> String {
    format!(
        "{}:{}",
        from_config("DASHBOARD_EVENTS_CHANNEL").as_str(),
        dashboard_name.as_ref()
    )
}


/// Returns JSON with inserted `tile_id` at `"tile-id"`
fn payload_with_tile_id(
    mut tile_data: serde_json::Value,
    tile_id: &str,
) -> Result<String, &'static str> {
    let mut tile_obj = match tile_data.as_object_mut() {
        None => return Err("Payload is not an object"),
        Some(v) => v,
    };
    tile_obj.insert(
        String::from("tile-id"),
        serde_json::Value::String(String::from(tile_id)),
    );
    match serde_json::to_string::<serde_json::Map<String, serde_json::Value>>(tile_obj) {
        Err(_) => Err("Failed converting to JSON"),
        Ok(v) => Ok(v),
    }
}



impl Db {
    pub fn new() -> Result<Db, &'static str> {
        // TODO: get from thread pool
        let connection = get_redis_con(from_config("DASHBOARD_REDIS_IP_PORT").as_str())?;
        let db = Db { connection: connection };
        Ok(db)
    }

    #[cfg(test)]
    pub fn run_cmd(&self, cmd: &str) -> Result<(), Box<Error>> {
        Ok(redis::cmd(cmd).query(&self.connection)?)
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
        self.upsert_dashboard(dashboard)
    }

    /// Inserts `dashboard` (or update when already exists)
    pub fn upsert_dashboard(&self, dashboard: &Dashboard) -> Result<(), String> {
        let json = serde_json::to_string(&dashboard)
            .map_err(|e| e.to_string())?;
        self.connection
            .hset::<_, _, _, u64>(DASHBOARDS_KEY, &dashboard.name, &json)
            .map(|_| ())
            .map_err(|e| e.to_string())
    }

    /// Returns sorted vector of `Dashboards`
    pub fn get_dashboards(&self) -> Result<Vec<Dashboard>, Box<Error>> {
        let mut collection: Vec<Dashboard> = self.connection
            .hscan::<_, (String, Dashboard)>(DASHBOARDS_KEY)?
            .map(|(_, dashboard_data)| dashboard_data)
            .collect();
        collection.sort_by(|a, b| natord::compare(&a.name, &b.name));
        Ok(collection)
    }

    /// Returns `Dashboard` saved at `dashboard_name`
    pub fn get_dashboard(&self, dashboard_name: &str) -> Result<Option<Dashboard>, Box<Error>> {
        let json_op = self.connection
            .hget::<_, _, Option<String>>(DASHBOARDS_KEY, dashboard_name)?;
        let json = match json_op {
            None => return Ok(None),
            Some(j) => j,
        };
        let dashboard: Dashboard = serde_json::from_str(&json)?;
        Ok(Some(dashboard))
    }

    /// Deletes `Dashboard` at `dashboard_name`
    #[cfg(test)]
    pub fn delete_dashboard(&self, dashboard_name: &str) -> Result<(u64), String> {
        self.connection
            .hdel::<_, _, u64>(DASHBOARDS_KEY, dashboard_name)
            .map(|v| v)
            .map_err(|e| e.to_string())
    }

    pub fn get_tile_space(&self, dashboard_name: &str, tile_id: &str) -> String {
        format!("{}:{}", dashboard_name, tile_id)
    }

    /// Returns `tile` data for `tile_id` from `dashboard_name`
    pub fn get_tile(
        &self,
        dashboard_name: &str,
        tile_id: &str,
    ) -> Result<Option<String>, Box<Error>> {
        let json = self.connection
            .hget::<_, _, Option<String>>(TILES_KEY, self.get_tile_space(dashboard_name, tile_id))?;
        Ok(json)
    }

    /// Adds `tile_json` at `tile_id` for `dashboard_name` (or update if already exists)
    pub fn upsert_tile(
        &self,
        dashboard_name: &str,
        tile_id: &str,
        tile_json: &str,
    ) -> Result<(), Box<Error>> {
        let space = format!("{}:{}", dashboard_name, tile_id);
        let tile_data = serde_json::from_str::<serde_json::Value>(tile_json)?;
        let tile_json_with_id: String = payload_with_tile_id(tile_data, tile_id)?;
        self.connection
            .hset::<_, _, _, u64>(TILES_KEY, &space, tile_json_with_id)?;
        self.connection
            .publish::<_, _, ()>(get_dashboard_channel(dashboard_name), tile_id)?;
        Ok(())
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
    #[cfg(test)]
    pub fn new(name: String, owner_email: String, layout: String) -> Dashboard {
        let mut d = Dashboard {
            name: name,
            owner_email: owner_email,
            layout: layout,
            api_token: None,
        };
        d.assign_api_token();
        d
    }

    /// Generates and assigns api token
    pub fn assign_api_token(&mut self) {
        self.api_token = Some(Uuid::new_v4().to_string());
    }

    /// Returns api token
    pub fn get_api_token(&self) -> Option<&String> {
        self.api_token.as_ref()
    }
}

impl redis::FromRedisValue for Dashboard {
    fn from_redis_value(v: &redis::Value) -> redis::RedisResult<Dashboard> {
        match *v {
            redis::Value::Data(ref val) => {
                match serde_json::from_slice(val) {
                    Err(_) => Err((redis::ErrorKind::TypeError, "Can't unjson value").into()),
                    Ok(v) => Ok(v),
                }
            }
            _ => Err(
                (
                    redis::ErrorKind::ResponseError,
                    "Response type not Dashboard compatible.",
                ).into(),
            ),
        }
    }
}
