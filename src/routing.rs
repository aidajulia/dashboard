use gui_api;
use iron::Chain;
use mount::Mount;
use rest_api;
use staticfile::Static;
use std::path::Path;
use templating;
use utils::from_config;
use views;


pub fn get_mount() -> Mount {
    let mut views_chain = Chain::new(views::views_router());
    views_chain.link_after(templating::init_templating());

    let mut rest_chain = Chain::new(rest_api::rest_router());
    rest_chain.link_before(rest_api::AuthToken);

    let mut mount = Mount::new();
    mount
        .mount(
            "/static",
            Static::new(Path::new(from_config("DASHBOARD_STATIC_PATH").as_str())),
        )
        .mount("/gui-api/", gui_api::get_router())
        .mount("/api", rest_chain)
        .mount("/", views_chain);
    mount
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyper::header::ContentType;
    use iron::{Headers, status};
    use iron::headers::Authorization;
    use iron::mime;
    use iron::prelude::*;
    use iron_test::{request, response};
    use std::error::Error;
    use utils::*;

    fn _get_data(url: &str) -> Response {
        let mut headers = Headers::new();
        headers.set(Authorization("change-me".to_owned()));
        request::get(url, headers, &get_mount()).unwrap()
    }

    fn _post_data(data: &str) -> Response {
        let mut headers = Headers::new();
        headers.set(Authorization("change-me".to_owned()));
        request::post(
            "http://localhost:8000/api/tile/tile_id",
            headers,
            data,
            &get_mount(),
        ).unwrap()
    }

    fn assert_html(response: Response) {
        assert_eq!(
            response.headers.get::<ContentType>().unwrap().0,
            ContentType::html().0
        );
    }

    fn assert_json(response: Response) {
        assert_eq!(
            response.headers.get::<ContentType>().unwrap().0,
            ContentType::json().0
        );
    }

    #[test]
    fn dashboard_show_returns_200() {
        load_config(None);

        let response = request::get("http://localhost:8000/", Headers::new(), &get_mount())
            .unwrap();
        assert_eq!(response.status.unwrap(), status::Ok);
        assert_html(response);
    }

    #[test]
    fn tile_get_returns_200() {
        load_config(None);
        _post_data("{}");

        let response = _get_data("http://localhost:8000/api/tile/tile_id");

        assert_eq!(response.status.unwrap(), status::Ok);
        assert_json(response);
    }

    #[test]
    fn tile_get_returns_404_when_tile_id_is_missing() {
        load_config(None);

        let response = _get_data("http://localhost:8000/api/tile/missing");

        assert_eq!(response.status.unwrap(), status::NotFound);
        assert_json(response);
    }


    #[test]
    fn tile_post_returns_201() {
        load_config(None);

        let response = _post_data("{}");

        assert_eq!(response.status.unwrap(), status::Created);
        assert_json(response);
    }

    #[test]
    fn tile_post_saves_tile_id_in_tile_data() {
        load_config(None);
        let tile_data = "{}";

        let response = _post_data(tile_data);

        assert_eq!(response.status.unwrap(), status::Created);
        let json = response::extract_body_to_string(response);
        assert_eq!(json, "{\"tile-id\":\"tile_id\"}");
    }

    #[test]
    fn tile_post_returns_400_when_json_invalid() {
        load_config(None);

        let response = _post_data("{,}");

        assert_eq!(response.status.unwrap(), status::BadRequest);
        assert_json(response);
    }

    #[test]
    fn static_url_gives_200() {
        load_config(None);

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
    fn api_gives_ok_when_tokens_machted() {
        load_config(None);

        let mut headers = Headers::new();
        headers.set(Authorization("change-me".to_owned()));
        let response = request::get(
            "http://localhost:8000/api/tile/tile_id",
            headers,
            &get_mount(),
        );

        assert_eq!(response.is_ok(), true);
    }

    #[test]
    fn api_gives_err_when_token_missing() {
        load_config(None);

        let response = request::get("http://localhost:8000/api", Headers::new(), &get_mount());

        let error = response.err().unwrap();
        assert_eq!(error.response.status.unwrap(), status::Forbidden);
        assert_eq!(error.description(), "Token missing");
    }

    #[test]
    fn api_gives_err_when_token_is_different() {
        load_config(None);

        let mut headers = Headers::new();
        headers.set(Authorization("unmatched-token".to_owned()));
        let response = request::get(
            "http://localhost:8000/api/tile/tile_id",
            headers,
            &get_mount(),
        );

        let error = response.err().unwrap();
        assert_eq!(error.response.status.unwrap(), status::Forbidden);
        assert_eq!(error.description(), "Tokens unmatched");
    }
}
