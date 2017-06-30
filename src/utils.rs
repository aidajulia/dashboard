use dotenv;
use iron::headers::ContentType;
use iron::prelude::*;
use iron::status::Status;
use redis;
use redis::Connection;
use std::clone::Clone;
use std::env;
use std::iter::Iterator;
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


pub fn get_page_items<T: Clone + Iterator>(
    iter: T,
    page_number: u64,
    per_page: u64,
) -> Result<(Vec<<T as Iterator>::Item>, usize), String> {
    if page_number < 1 {
        return Err("Page number must be 1 or greater".to_string());
    }
    let per_page = per_page as usize;
    let page_number = (page_number - 1) as usize;
    let items_count = iter.clone().count();

    let max_page = if items_count % per_page > 0 {
        (items_count / per_page) + 1
    } else {
        items_count / per_page
    };
    if page_number > max_page {
        return Err(format!(
            "Page doesn't exist. Page number should be at most: {}",
            max_page
        ));
    }

    let to_skip = page_number * per_page;
    let collection = iter.enumerate()
        .map(|(_, v)| v)
        .skip(to_skip)
        .take(per_page)
        .collect();
    Ok((collection, max_page))
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

        get_redis_con(from_config("DASHBOARD_REDIS_IP_PORT").as_str()).unwrap();
    }

    #[test]
    fn get_page_items_works_ok() {
        let iter = "0123456".chars();

        let items = get_page_items(iter, 2, 2);

        assert_eq!(items, Ok((vec!['2', '3'], 4)));
    }

    #[test]
    fn get_page_items_has_max_1_when_2_items() {
        let iter = "12".chars();

        let items = get_page_items(iter, 1, 2);

        assert_eq!(items, Ok((vec!['1', '2'], 1)));
    }

    #[test]
    fn get_page_items_result_err_when_page_out_of_range() {
        let iter = "1234567".chars();

        let items = get_page_items(iter, 100, 2);

        assert_eq!(
            items,
            Err(
                "Page doesn\'t exist. Page number should be at most: 4".to_string(),
            )
        );

    }

}
