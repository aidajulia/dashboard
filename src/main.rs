extern crate bodyparser;
extern crate persistent;
extern crate clap;
extern crate dotenv;
extern crate handlebars;
extern crate handlebars_iron as hbs;
extern crate hyper;
extern crate iron;
#[cfg(test)]
extern crate iron_test;
extern crate mount;
extern crate redis;
extern crate router;
extern crate serde;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate slog;
#[macro_use]
extern crate slog_scope;
extern crate slog_term;
extern crate staticfile;
extern crate uuid;
extern crate ws;

use clap::*;
use iron::prelude::*;
use slog::Drain;


mod db;
mod gui_api;
mod rest_api;
mod routing;
mod templating;
mod utils;
mod views;
mod websocket;
use hyper::server::Listening;
use routing::get_mount;
use utils::{from_config, load_config};
use websocket::run_ws_listener;

fn run_http_listener(ip_port: String) -> Listening {
    println!("Serving HTTP on: {}", ip_port);
    Iron::new(get_mount())
        .http(ip_port.as_str())
        .expect("starting HTTP server FAILED")
}

fn setup_logger() -> slog::Logger {
    let decorator = slog_term::PlainSyncDecorator::new(std::io::stdout());
    let drain = slog_term::FullFormat::new(decorator).build().fuse();
    slog::Logger::root(drain, slog_o!())
}

fn main() {
    let _guard = slog_scope::set_global_logger(setup_logger());
    debug!("Logger registered..");

    // cli args
    let matches = app_from_crate!()
        .arg(
            Arg::with_name("config-path")
                .help(
                    "Path to .env file (see https://github.com/slapresta/rust-dotenv)",
                )
                .default_value("dashboard.env")
                .takes_value(true)
                .short("c"),
        )
        .get_matches();
    let config_path = matches.value_of("config-path");
    load_config(config_path);

    // http listener
    let http_ip_port = from_config("DASHBOARD_IP_PORT");
    let _listener = run_http_listener(http_ip_port);


    // websocket listener
    let ws_ip_port = from_config("DASHBOARD_WEBSOCKET_IP_PORT");
    run_ws_listener(ws_ip_port);
    // unreachable code
}
