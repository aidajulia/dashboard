use hyper::header::Authorization;
use iron;
use iron::prelude::*;
use iron::status;
use iron::status::Status;
use redis::Commands;
use router::Router;
use serde_json;
use serde_json::Value;
use std::error::Error;
use std::fmt;
use std::io::Read;
use utils::{get_redis_con, from_config, json_response};


pub fn rest_router() -> Router {
    let mut router = Router::new();
    router.get("/tile/:tile_id", tile_get, "tile_get");
    router.post("/tile/:tile_id", tile_post, "tile_post");
    router
}

pub struct AuthToken;

#[derive(Debug)]
struct StringError(String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        &*self.0
    }
}

impl iron::BeforeMiddleware for AuthToken {
    fn before(&self, request: &mut Request) -> IronResult<()> {
        let dashboard_token = from_config("DASHBOARD_DASHBOARD_TOKEN");
        let request_token = match request.headers.get::<Authorization<String>>() {
            Some(v) => v,
            None => {
                return Err(IronError::new(
                    StringError("Token missing".to_string()),
                    status::Forbidden,
                ))
            }
        };
        if request_token.0 == dashboard_token {
            Ok(())
        } else {
            Err(IronError::new(
                StringError("Tokens unmatched".to_string()),
                status::Forbidden,
            ))
        }
    }
}

/// Returns String with inserted `tile_id` at `"tile-id"` or error
fn payload_with_tile_id(mut tile_data: Value, tile_id: &str) -> Result<String, &str> {
    let mut tile_obj = match tile_data.as_object_mut() {
        None => return Err("Payload is not an object"),
        Some(v) => v,
    };
    tile_obj.insert(
        String::from("tile-id"),
        Value::String(String::from(tile_id)),
    );
    match serde_json::to_string::<serde_json::Map<String, serde_json::Value>>(tile_obj) {
        Err(_) => return Err("Failed converting to JSON"),
        Ok(v) => Ok(v),
    }
}

pub fn tile_get(req: &mut Request) -> IronResult<Response> {
    let tile_id = req.extensions
        .get::<Router>()
        .unwrap()
        .find("tile_id")
        .unwrap();
    let con = match get_redis_con(from_config("DASHBOARD_REDIS_IP_HOST").as_str()) {
        Ok(v) => v,
        Err(e) => return json_response(Status::InternalServerError, e),
    };
    match con.get::<_, String>(tile_id) {
        Err(_) => json_response(Status::NotFound, ""),
        Ok(val) => json_response(Status::Ok, val.as_str()),
    }
}

pub fn tile_post(req: &mut Request) -> IronResult<Response> {
    let tile_id = req.extensions
        .get::<Router>()
        .unwrap()
        .find("tile_id")
        .unwrap();

    let mut payload = String::new();
    if let Err(e) = req.body.read_to_string(&mut payload) {
        return json_response(
            Status::InternalServerError,
            &format!("Reading payload FAILED ({})", e),
        );
    }
    let tile_data = match serde_json::from_str::<Value>(&payload) {
        Err(_) => return json_response(Status::BadRequest, "Invalid JSON"),
        Ok(val) => val,
    };
    let payload_with_id: String = match payload_with_tile_id(tile_data, tile_id) {
        Err(e) => return json_response(Status::InternalServerError, e),
        Ok(v) => v,
    };
    let con = match get_redis_con(from_config("DASHBOARD_REDIS_IP_HOST").as_str()) {
        Ok(v) => v,
        Err(e) => return json_response(Status::InternalServerError, e),
    };
    if con.set::<_, _, ()>(tile_id, &payload_with_id).is_err() {
        return json_response(Status::InternalServerError, "Saving tile data FAILED");
    }
    if con.publish::<_, _, ()>(from_config("DASHBOARD_EVENTS_CHANNEL").as_str(), tile_id)
        .is_err()
    {
        return json_response(Status::InternalServerError, "Publishing tile data FAILED");
    }
    json_response(Status::Created, &payload_with_id)
}
