use db;
use handlebars::to_json;
use hbs::Template;
use iron::middleware;
use iron::prelude::*;
use iron::status;
use params;
use router::Router;
use serde_json::value::{Value, Map};
use std::str::FromStr;
use templating;
use utils::{from_config, get_page_items};


pub fn get_handler() -> middleware::Chain {
    let mut views_chain = Chain::new(views_router());
    views_chain.link_after(templating::init_templating());
    views_chain
}

fn views_router() -> Router {
    let mut router = Router::new();
    router.get("/", dashboard_list, "home");
    router.get("/dashboard/new", dashboard_new, "dashboard_new");
    router.get("/dashboard/list", dashboard_list, "dashboard_list");
    router.get(
        "/dashboard/show/:dashboard_name",
        dashboard_show,
        "dashboard_show",
    );
    router
}


fn dashboard_show_data(dashboard: &db::Dashboard) -> Map<String, Value> {
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
    data.insert("dashboard_name".to_string(), to_json(&dashboard.name));
    data
}


pub fn dashboard_show(req: &mut Request) -> IronResult<Response> {
    let dashboard_name = req.extensions
        .get::<Router>()
        .unwrap()
        .find("dashboard_name")
        .unwrap_or("demo");

    let db = match db::Db::new() {
        Err(e) => return Ok(Response::with((status::InternalServerError, e.to_string()))),
        Ok(d) => d,
    };
    let dashboard = match db.get_dashboard(dashboard_name) {
        Err(e) => return Ok(Response::with((status::InternalServerError, e.to_string()))),
        Ok(None) => return Ok(Response::with((status::NotFound, "Dashboard missing"))),
        Ok(Some(d)) => d,
    };
    let template = Template::new(
        &format!("dashboards/{}", &dashboard.layout),
        dashboard_show_data(&dashboard),
    );
    Ok(Response::with((status::Ok, template)))
}

pub fn dashboard_new(_req: &mut Request) -> IronResult<Response> {
    let mut resp = Response::new();
    resp.set_mut(Template::new("dashboard-new", Map::new()))
        .set_mut(status::Ok);
    Ok(resp)

}

pub fn dashboard_list(req: &mut Request) -> IronResult<Response> {
    let map = match req.get_ref::<params::Params>() {
        Err(e) => return Ok(Response::with((status::InternalServerError, e.to_string()))),
        Ok(v) => v,
    };
    let err_msg = "Come back later, we encounter an error".to_string();
    let page_current = match map.find(&["page"]) {
        Some(&params::Value::String(ref v)) => {
            match v.parse() {
                Err(_) => {
                    return Ok(Response::with((
                        status::Ok,
                        format!("Page should be a positive number ({})", v),
                    )))
                }
                Ok(p) => p,
            }
        }
        _ => 1,
    };
    let mut tmplt_data = Map::new();

    let db = match db::Db::new() {
        Err(e) => return Ok(Response::with((status::InternalServerError, e.to_string()))),
        Ok(d) => d,
    };
    let per_page = 12; //TODO: user settings
    let dashboards = match db.get_dashboards() {
        Err(_) => return Ok(Response::with((status::InternalServerError, err_msg))),
        Ok(v) => v,
    };
    let (dashboards, page_max) = match get_page_items(dashboards.iter(), page_current, per_page) {
        Err(e) => return Ok(Response::with((status::InternalServerError, e.to_string()))),
        Ok(v) => v,
    };
    tmplt_data.insert("page-current".to_string(), to_json(&(page_current)));
    tmplt_data.insert("page-max".to_string(), to_json(&page_max));
    tmplt_data.insert("show-pagination".to_string(), to_json(&(page_max > 1)));
    tmplt_data.insert("dashboards".to_string(), to_json(&dashboards));
    Ok(Response::with(
        (status::Ok, Template::new("dashboard-list", tmplt_data)),
    ))
}


#[cfg(test)]
mod tests {
    use super::*;
    use db;
    use iron::Headers;
    use iron::status::Status;
    use iron_test::{request, response};
    use utils;

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
            &get_handler(),
        ).unwrap();

        assert_eq!(resp.status, Some(Status::Ok));
        let body = response::extract_body_to_string(resp);
        assert_eq!(body.contains(&dashboard.name), true);
    }

    #[test]
    fn test_dashboard_list_works_when_dashboards_count_0() {
        utils::load_config(None);
        let db = db::Db::new().unwrap();
        db.run_cmd("flushall").unwrap();

        let resp = request::get(
            "http://localhost:3000/dashboard/list",
            Headers::new(),
            &get_handler(),
        ).unwrap();

        assert_eq!(resp.status, Some(Status::Ok));
        let body = response::extract_body_to_string(resp);
        assert_eq!(body.contains("No Dashboard created yet"), true);
    }

    #[test]
    fn test_dashboard_list_shows_13th_elem_on_second_page() {
        utils::load_config(None);
        let db = db::Db::new().unwrap();
        db.run_cmd("flushall").unwrap();

        for idx in 0..14 {
            let dashboard_name = format!("dashboard-{}", idx);
            let dashboard = db::Dashboard::new(
                dashboard_name,
                "login@email.com".to_string(),
                "2x4".to_string(),
            );
            db.create_dashboard(&dashboard).unwrap();
        }

        let resp = request::get(
            "http://localhost:3000/dashboard/list?page=2",
            Headers::new(),
            &get_handler(),
        ).unwrap();

        assert_eq!(resp.status, Some(Status::Ok));
        let body = response::extract_body_to_string(resp);
        assert_eq!(body.contains("dashboard-13"), true);
    }
}
