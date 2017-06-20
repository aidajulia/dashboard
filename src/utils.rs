use dotenv;
use iron::headers::ContentType;
use iron::prelude::*;
use iron::status::Status;
use redis;
use redis::Connection;
use std::env;
use std::path::Path;

pub fn redis_url(ip_port: &str) -> String {
    format!("redis://{}/", ip_port)
}

pub fn get_redis_con(ip_port: &str) -> Result<Connection, &'static str> {
    // TODO: use redis pool: https://github.com/darayus/iron-redis-middleware
    // TODO: ip_port as ref?
    let redis_url = redis_url(ip_port);
    let client = redis::Client::open(redis_url.as_str())
        .expect(format!("Can't connect to: {}", redis_url).as_str());
    client.get_connection().map_err(|_| "Can't connect redis")
}

pub fn load_config(config_path: Option<&str>) {
    dotenv::from_path(Path::new(config_path.unwrap_or("dashboard.env"))).expect(
        format!("Loading config: \"{:?}\" FAILED", config_path).as_str(),
    )
}

pub fn from_config(key: &str) -> String {
    env::var(&key).expect(
        format!("Set value for key: \"{}\" in config.", &key).as_str(),
    )
}

pub fn json_response(status: Status, payload: &str) -> IronResult<Response> {
    Ok(Response::with((ContentType::json().0, status, payload)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn load_config_works_when_path_is_passed() {
        let mut config_path = env::current_dir().unwrap();
        config_path.push("dashboard.env");

        load_config(config_path.to_str());

        assert_eq!(from_config("DASHBOARD_IP_PORT").as_str(), "0.0.0.0:8000");
    }

    #[test]
    fn load_config_works_when_path_is_none() {
        load_config(None);

        assert_eq!(from_config("DASHBOARD_IP_PORT").as_str(), "0.0.0.0:8000");
    }

    #[test]
    fn redis_connections_works() {
        load_config(None);

        get_redis_con(from_config("DASHBOARD_REDIS_IP_HOST").as_str()).unwrap();
    }

}
