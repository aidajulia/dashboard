use handlebars::to_json;
use hbs::Template;
use iron::prelude::*;
use iron::status;
use router::Router;
use serde_json::value::{Value, Map};
use std::str::FromStr;
use utils::from_config;


pub fn views_router() -> Router {
    let mut router = Router::new();
    //TODO:: random dashboard?
    router.get("/", dashboard_show, "home");
    router.get("/dashboard/new", dashboard_new, "dashboard_new");
    router.get(
        "/dashboard/show/:dashboard_name",
        dashboard_show,
        "dashboard_show",
    );
    router
}


fn dashboard_show_data() -> Map<String, Value> {
    let mut data = Map::new();
    let ssl_enable = FromStr::from_str(&from_config("DASHBOARD_WEBSOCKET_SSL"))
        .expect("DASHBOARD_WEBSOCKET_SSL is not bool");
    let scheme = if ssl_enable {
        String::from("wss")
    } else {
        String::from("ws")
    };
    let websocket_uri = format!(
        "{}://{}",
        scheme,
        from_config("DASHBOARD_FRONT_WEBSOCKET_IP_PORT")
    );
    data.insert("websocket_uri".to_string(), to_json(&websocket_uri));
    data
}

pub fn dashboard_show(req: &mut Request) -> IronResult<Response> {
    let dashboard_name = req.extensions
        .get::<Router>()
        .unwrap()
        .find("dashboard_name")
        .unwrap_or("2x8");
    let mut resp = Response::new();
    // TODO: file or 404
    resp.set_mut(Template::new(dashboard_name, dashboard_show_data()))
        .set_mut(status::Ok);
    Ok(resp)
}

pub fn dashboard_new(_req: &mut Request) -> IronResult<Response> {
    let mut resp = Response::new();
    resp.set_mut(Template::new("dashboard-new", Map::new()))
        .set_mut(status::Ok);
    Ok(resp)

}
