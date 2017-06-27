use gui_api;
use iron::Chain;
use mount::Mount;
use rest_api;
use staticfile::Static;
use std::path::Path;
use utils;
use views;


pub fn get_mount() -> Mount {
    let views_handler = views::get_handler();

    let mut rest_chain = Chain::new(rest_api::rest_router());
    rest_chain.link_before(rest_api::AuthToken);

    let mut mount = Mount::new();
    mount
        .mount(
            "/static",
            Static::new(Path::new(
                utils::from_config("DASHBOARD_STATIC_PATH").as_str(),
            )),
        )
        .mount("/gui-api/", gui_api::get_router())
        .mount("/api", rest_chain)
        .mount("/", views_handler);
    mount
}

#[cfg(test)]
mod tests {
    use super::*;
    use db;
    use hyper::header::ContentType;
    use iron::{Headers, status};
    use iron::headers::Authorization;
    use iron::mime;
    use iron::prelude::*;
    use iron::status::Status;
    use iron_test::{request, response};
    use std::error::Error;
    use test_utils;


    fn _get_data(url: &str) -> Response {
        let mut headers = Headers::new();
        headers.set(Authorization("change-me".to_owned()));
        request::get(url, headers, &get_mount()).unwrap()
    }

    fn _post_data(url: String, api_key: &str, data: &str) -> Response {
        let mut headers = Headers::new();
        headers.set(Authorization(api_key.to_owned()));
        request::post(&url, headers, data, &get_mount()).unwrap()
    }

    fn assert_json(response: Response) {
        assert_eq!(
            response.headers.get::<ContentType>().unwrap().0,
            ContentType::json().0
        );
    }

    #[test]
    fn dashboard_show_returns_200() {
        utils::load_config(None);

        let response = request::get("http://localhost:8000/", Headers::new(), &get_mount())
            .unwrap();
        assert_eq!(response.status.unwrap(), status::Ok);
        assert_eq!(
            response.headers.get::<ContentType>().unwrap().0,
            ContentType::html().0
        );
    }

    #[test]
    fn test_dashboard_shows_created_dashboard() {
        utils::load_config(None);
        let dashboard_name = "Uber-dashboard-name".to_string();
        let db = db::Db::new().unwrap();
        db.delete_dashboard(&dashboard_name).unwrap();
        let dashboard = db::Dashboard::new(
            dashboard_name,
            "login@email.com".to_string(),
            "2x4".to_string(),
        );
        db.create_dashboard(&dashboard).unwrap();

        let resp = request::get(
            &format!("http://localhost:3000/dashboard/show/{}", dashboard.name),
            Headers::new(),
            &get_mount(),
        ).unwrap();

        assert_eq!(resp.status, Some(Status::Ok));
        let body = response::extract_body_to_string(resp);
        assert_eq!(body.contains(&dashboard.name), true);
    }

    fn tile_get_setup() {
        utils::load_config(None);
        let db = db::Db::new().unwrap();
        let dashboard_name = "dashboard-test";
        test_utils::upsert_dashboard(&db, &dashboard_name);
        db.upsert_tile(&dashboard_name, "tile-test", "{}").unwrap();
    }

    #[test]
    fn tile_get_returns_200() {
        tile_get_setup();

        let response = _get_data(
            "http://localhost:8000/api/dashboard/dashboard-test/tile/tile-test",
        );

        assert_eq!(response.status.unwrap(), status::Ok);
        assert_json(response);
    }

    #[test]
    fn tile_get_returns_404_when_dashboard_is_missing() {
        tile_get_setup();

        let response = _get_data(
            "http://localhost:8000/api/dashboard/dashboard-missing/tile/tile-test",
        );

        assert_eq!(response.status.unwrap(), status::NotFound);
        assert_json(response);
    }

    #[test]
    fn tile_get_returns_404_when_tile_is_missing() {
        tile_get_setup();

        let response = _get_data(
            "http://localhost:8000/api/dashboard/dashboard-test/tile/tile-missing",
        );

        assert_eq!(response.status.unwrap(), status::NotFound);
        assert_json(response);
    }

    #[test]
    fn tile_post_returns_201() {
        utils::load_config(None);
        let db = db::Db::new().unwrap();
        let dashboard = test_utils::upsert_dashboard(&db, "dashboard-test");
        let url = format!(
            "http://localhost:8000/api/dashboard/{}/tile/tile_id",
            dashboard.name
        );
        let api_key = dashboard.get_api_token().unwrap();

        let response = _post_data(url, api_key, "{}");

        assert_eq!(response.status.unwrap(), status::Created);
        assert_json(response);
    }

    #[test]
    fn tile_post_saves_tile_id_in_tile_data() {
        utils::load_config(None);
        let db = db::Db::new().unwrap();
        let dashboard = test_utils::upsert_dashboard(&db, "dashboard-test");
        let url = format!(
            "http://localhost:8000/api/dashboard/{}/tile/tile-test",
            dashboard.name
        );
        let api_key = dashboard.get_api_token().unwrap();
        let tile_data = "{}";

        let response = _post_data(url, api_key, &tile_data);

        assert_eq!(response.status.unwrap(), status::Created);
        let json = db.get_tile("dashboard-test", "tile-test").unwrap();
        assert_eq!(json, Some("{\"tile-id\":\"tile-test\"}".to_string()));
    }


    #[test]
    fn tile_post_returns_400_when_json_invalid() {
        utils::load_config(None);
        let db = db::Db::new().unwrap();
        let dashboard = test_utils::upsert_dashboard(&db, "dashboard-test");
        let url = format!(
            "http://localhost:8000/api/dashboard/{}/tile/tile_id",
            dashboard.name
        );
        let api_key = dashboard.get_api_token().unwrap();

        let response = _post_data(url, api_key, "{,}");

        assert_eq!(response.status.unwrap(), status::BadRequest);
        assert_json(response);
    }

    #[test]
    fn static_url_gives_200() {
        utils::load_config(None);

        let response = request::get(
            "http://localhost:8000/static/elements.html",
            Headers::new(),
            &get_mount(),
        ).unwrap();

        assert_eq!(response.status.unwrap(), status::Ok);
        assert_eq!(
            response.headers.get::<ContentType>().unwrap().0,
            ContentType(mime::Mime(
                mime::TopLevel::Text,
                mime::SubLevel::Html,
                vec![],
            )).0
        );
    }

    #[test]
    fn tile_post_gives_err_when_token_missing() {
        utils::load_config(None);

        let response = request::post(
            "http://localhost:8000/api/dashboard/dashboard-test/tile/tile-test",
            Headers::new(),
            "{}",
            &get_mount(),
        );

        let error = response.err().unwrap();
        assert_eq!(error.response.status.unwrap(), status::Forbidden);
        assert_eq!(error.description(), "Token missing");
    }

    #[test]
    fn tile_post_gives_err_when_token_is_different() {
        utils::load_config(None);
        let db = db::Db::new().unwrap();
        let dashboard_name = "dashboard-test";
        test_utils::upsert_dashboard(&db, &dashboard_name);
        db.upsert_tile(&dashboard_name, "tile-test", "{}").unwrap();
        let mut headers = Headers::new();
        headers.set(Authorization("incorrect-token".to_owned()));

        let response = request::post(
            "http://localhost:8000/api/dashboard/dashboard-test/tile/tile-test",
            headers,
            "{}",
            &get_mount(),
        ).unwrap();

        assert_eq!(response.status, Some(status::Forbidden));
        let body = response::extract_body_to_string(response);
        assert_eq!(body, "Tokens unmatched");
    }
}
