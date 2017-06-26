use db;
use hyper::header::Authorization;
use iron;
use iron::prelude::*;
use iron::status;
use iron::status::Status;
use router::Router;
use serde_json;
use std::error::Error;
use std::fmt;
use std::io::Read;
use utils::json_response;


pub fn rest_router() -> Router {
    let mut router = Router::new();
    router.get(
        "/dashboard/:dashboard_name/tile/:tile_id",
        tile_get,
        "tile_get",
    );
    router.post(
        "/dashboard/:dashboard_name/tile/:tile_id",
        tile_post,
        "tile_post",
    );
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


/// Returns token stored in authorization header
fn get_request_token(request: &Request) -> Result<String, IronError> {
    let token = request
        .headers
        .get::<Authorization<String>>()
        .ok_or_else(|| {
            IronError::new(StringError("Token missing".to_string()), status::Forbidden)
        })?
        .to_owned()
        .0;

    Ok(token)
}


impl iron::BeforeMiddleware for AuthToken {
    fn before(&self, request: &mut Request) -> IronResult<()> {
        match request.headers.get::<Authorization<String>>() {
            None => {
                Err(IronError::new(
                    StringError("Token missing".to_string()),
                    status::Forbidden,
                ))
            }
            Some(_) => Ok(()),
        }
    }
}

pub fn tile_get(req: &mut Request) -> IronResult<Response> {
    let (dashboard_name, tile_id) = {
        let router = req.extensions.get::<Router>().unwrap();

        (
            router.find("dashboard_name").unwrap(),
            router.find("tile_id").unwrap(),
        )
    };
    let db = match db::Db::new() {
        Err(e) => return json_response(Status::InternalServerError, &e.to_string()),
        Ok(v) => v,
    };
    match db.get_tile(dashboard_name, tile_id) {
        Err(e) => json_response(Status::InternalServerError, &e.to_string()),
        Ok(None) => json_response(Status::NotFound, ""),
        Ok(Some(val)) => json_response(Status::Ok, val.as_str()),
    }
}

fn _tile_post(req: &mut Request) -> Result<(Status, String), Box<Error>> {
    let (dashboard_name, tile_id) = {
        let router = req.extensions.get::<Router>().unwrap();
        (
            router.find("dashboard_name").unwrap(),
            router.find("tile_id").unwrap(),
        )
    };
    let request_token = get_request_token(req)?;
    let db = db::Db::new()?;
    let dashboard = match db.get_dashboard(dashboard_name)? {
        None => return Ok((Status::NotFound, "Dashboard doesn't exist".to_string())),
        Some(v) => v,
    };
    let dashboard_api_token = match dashboard.get_api_token() {
        None => {
            return Ok((
                Status::InternalServerError,
                "Dashboard doens't have API Token, contact webpage administrators".to_string(),
            ))
        }
        Some(v) => v,
    };
    if &request_token != dashboard_api_token {
        return Ok((Status::Forbidden, "Tokens unmatched".to_string()));
    }

    let mut json = String::new();
    if let Err(e) = req.body.read_to_string(&mut json) {
        return Ok((
            Status::InternalServerError,
            format!("Reading payload FAILED ({})", e),
        ));
    }

    if let Err(e) = serde_json::from_str::<serde_json::Value>(&json) {
        return Ok((
            Status::BadRequest,
            format!("Unable to unjson payload: ({})", e),
        ));
    }
    db.upsert_tile(dashboard_name, tile_id, &json)?;
    Ok((Status::Created, "".to_string()))
}

pub fn tile_post(req: &mut Request) -> IronResult<Response> {
    let (status, msg) = match _tile_post(req) {
        Err(_) => return json_response(Status::InternalServerError, "We're working on fix"),
        Ok(v) => v,
    };
    json_response(status, &msg)
}
