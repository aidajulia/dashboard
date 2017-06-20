use bodyparser;
use iron::headers::ContentType;
use iron::prelude::*;
use iron::status::Status;
use persistent;
use router::Router;
use serde_json;
use utils::{json_response};

use db;
use db::{Dashboard};


const MAX_BODY_LENGTH: usize = 1024 * 1024 * 10;


#[derive(Serialize)]
struct ResponseMessage {
    message: String,
}


fn json_response_as_msg<M: AsRef<str>>(status: Status, details_msg: M) -> IronResult<Response> {
    let msg = match status {
        Status::Created => details_msg.as_ref().to_string(),
        _ => format!("Couldn't create dashboard ({})", details_msg.as_ref()),
    };
    let resp_msg = ResponseMessage{ message: msg };
    let json = serde_json::to_string(&resp_msg).unwrap_or("{}".to_string());
    json_response(status, &json)
}


pub fn get_router() -> Chain {
    let mut router = Router::new();
    router.post("/dashboard", dashboard_post, "dashboard_post");
    let mut chain = Chain::new(router);
    chain.link_before(persistent::Read::<bodyparser::MaxBodyLength>::one(MAX_BODY_LENGTH));
    chain
}


pub fn dashboard_post(req: &mut Request) -> IronResult<Response> {
    let mut dashboard = match req.get::<bodyparser::Struct<Dashboard>>() {
        Err(e) => return json_response_as_msg(Status::BadRequest, e.to_string()),
        Ok(None) => return json_response_as_msg(Status::BadRequest, "Payload is missing"),
        Ok(Some(v)) => v,
    };

    let db = match db::Db::new() {
        Err(e) => return json_response_as_msg(Status::InternalServerError, e.to_string()),
        Ok(v) => v,
    };

    dashboard.assign_api_token();
    let api_token = match dashboard.get_api_token() {
        None => return json_response_as_msg(Status::InternalServerError, "Can't generate token"),
        Some(v) => v,
    };

    if let Err(e) = db.create_dashboard(&dashboard) {
        return json_response_as_msg(Status::BadRequest, e.to_string());
    }

    // TODO:: 
    //if let Err(e) = notification.send_token(api_token=Dashboard.api_token, email=email) {
    //    if Err(e) = Dashboard.delete() {
    //        let msg = "Couldn't send api-token to email: {} and failure cleanups also failed. Please contact {} to fix it";
    //    }
    //    let msg = "Couldn't send api-token to email: {}. Create dashboard later";
    //}

    json_response_as_msg(Status::Created, format!("Dashboard is created! Save your personal token: {}", api_token))
}


#[cfg(test)]
mod tests {
    use super::*;
    use iron::{Headers};
    use iron_test::{request, response};
    use db;
    use utils;
    
    #[test]
    fn test_dashboard_post_creates_dashboard_when_ok() {
        utils::load_config(None);
        // TODO: no setup/teardown bundled feature, move when possible
        let db = db::Db::new().unwrap();
        db.delete_dashboard("some-name").unwrap();
        let payload = "{\"name\":\"some-name\",\"owner_email\":\"some-dude@some-email.com\",\"layout\":\"single-tile\"}";
        let mut headers = Headers::new();
        headers.set(ContentType::json());

        let resp = request::post("http://localhost:3000/dashboard",
                                 headers,
                                 &payload,
                                 &get_router()).unwrap();

        assert_eq!(resp.status, Some(Status::Created));
        let body = response::extract_body_to_string(resp);
        let json = db.get_dashboard("some-name").unwrap().unwrap();
        let dashboard: Dashboard = serde_json::from_str(&json).unwrap();
        assert_eq!(&dashboard.owner_email, "some-dude@some-email.com");
        let expected = format!("{{\"message\":\"Dashboard is created! Save your personal token: {}\"}}", dashboard.get_api_token().unwrap());
        assert_eq!(body, expected);
    }
}