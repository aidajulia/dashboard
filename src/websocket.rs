use db;
use redis;
use serde_json;
use std::error::Error;
use std::thread;
use utils::{from_config, redis_url};
use ws;


// TODO: rewrite it! what would i think of? xD
// TODO: rm val_or_send_msg_err, make functions which return error and use
// case analysis to convert (or better implement From) it to to user/client etc.

macro_rules! val_or_send_msg_err {
    ($expr: expr, $ws_server: ident, $err_msg: expr) => (match $expr {
        Ok(val) => val,
        Err(e) => {
            send_or_log_err(&$ws_server.out,
                            &Message {
                                kind: String::from("error"),
                                text: format!("{} ({})", $err_msg, e),
                            });
            return Ok(())
        }
    })
}

pub fn run_ws_listener(ip_port: &str) {
    println!("Serving websocket on: {}", ip_port);
    ws::listen(ip_port, |out| {
        Server {
            out: out,
            redis_url: redis_url(from_config("DASHBOARD_REDIS_IP_PORT").as_str()),
            dashboard_name: None,
        }
    }).expect("starting websocket FAILED");
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    //TODO: replace String with &str?
    kind: String,
    text: String,
}

fn send_err<M: AsRef<str>>(sender: &ws::Sender, msg: M) {
    let msg = Message {
        kind: "error".to_string(),
        text: msg.as_ref().to_string(),
    };
    send_or_log_err(sender, &msg);
}

fn send_or_log_err(sender: &ws::Sender, msg: &Message) {
    match serde_json::to_string(&msg) {
        Ok(as_str) => {
            debug!("sending: {}", as_str);
            sender.send(as_str).unwrap_or_else(|msg| {
                debug!("sending msg: '{}' FAILED", msg);
            });
        }
        Err(e) => debug!("JSONizing msg: '{:?}' FAILED {}", msg, e),
    };
}

fn ws_msg2msg(ws_msg: &ws::Message) -> Result<Message, String> {
    match ws_msg.as_text() {
        Ok(json) => {
            match serde_json::from_str(json) {
                Ok(v) => Ok(v),
                Err(e) => Err(format!("FAILED unjsoning {} ({})", json, e)),
            }
        }
        Err(e) => Err(format!("FAILED converting to text {} ({})", ws_msg, e)),
    }
}


#[derive(Clone)]
pub struct Server {
    pub out: ws::Sender,
    pub redis_url: String,
    pub dashboard_name: Option<String>,
}

impl Server {
    fn publish_tiles(&mut self, tile_ids: Vec<String>, db: &db::Db) -> Result<(), Box<Error>> {
        for tile_id in &tile_ids {
            let dashboard_name = self.dashboard_name
                .clone()
                .ok_or_else(|| "Can't find Dashboard name".to_string())?;
            let (kind, text) = match db.get_tile(dashboard_name.as_str(), tile_id) {
                Err(e) => (
                    "error",
                    format!("FAILED getting value for {} ({})", tile_id, e),
                ),
                Ok(None) => ("error", "FAILED tile data doesn't exist".to_string()),
                Ok(Some(json)) => ("tile", json),
            };
            send_or_log_err(
                &self.out,
                &Message {
                    kind: String::from(kind),
                    text: text,
                },
            )
        }
        Ok(())
    }
}

impl ws::Handler for Server {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<(ws::Response)> {
        self.dashboard_name = {
            debug!("dashboard_name: {}", req.resource().to_owned());
            Some(
                req.resource()
                    .to_owned()
                    .trim_left_matches('/')
                    .trim_right_matches('/')
                    .to_string(),
            )
        };
        ws::Response::from_request(req)
    }

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        debug!("on_open");
        let redis_url = self.redis_url.clone();

        let dashboard_name = match self.dashboard_name.clone() {
            None => {
                send_or_log_err(
                    &self.out,
                    &Message {
                        kind: "error".to_string(),
                        text: "Can't find dashboard name".to_string(),
                    },
                );
                return Ok(());
            }
            Some(v) => v,
        };

        let channel_name = db::get_dashboard_channel(&dashboard_name);
        let cloned_server = self.clone();

        // spawn redis listener
        thread::spawn::<_, Result<(), ()>>(move || {
            debug!("thread open");

            let redis = val_or_send_msg_err!(
                redis::Client::open(redis_url.as_str()),
                cloned_server,
                format!("FAILED opening redis {}", redis_url)
            );
            let mut pubsub =
                val_or_send_msg_err!(redis.get_pubsub(), cloned_server, "FAILED getting pubsub");
            if let Err(e) = pubsub.subscribe(channel_name.as_str()) {
                send_or_log_err(
                    &cloned_server.out,
                    &Message {
                        kind: String::from("error"),
                        text: format!("can't subscribe {}", e),
                    },
                )
            };
            let db = match db::Db::new() {
                Err(_) => {
                    send_err(&cloned_server.out, "Can't get tile data");
                    return Ok(());
                }
                Ok(v) => v,
            };
            loop {
                let msg = val_or_send_msg_err!(
                    pubsub.get_message(),
                    cloned_server,
                    "FAILED getting published message"
                );
                let tile_id: String = val_or_send_msg_err!(
                    msg.get_payload(),
                    cloned_server,
                    "FAILED getting payload of published \
                     message"
                );

                let json = match db.get_tile(dashboard_name.as_str(), &tile_id) {
                    Err(_) => {
                        send_err(&cloned_server.out, "Can't get tile data");
                        return Ok(());
                    }
                    Ok(None) => {
                        send_err(
                            &cloned_server.out,
                            format!("Tile doesn't exists: {}", &tile_id),
                        );
                        return Ok(());
                    }
                    Ok(Some(v)) => v,
                };

                send_or_log_err(
                    &cloned_server.out,
                    &Message {
                        kind: String::from("tile"),
                        text: json,
                    },
                );
            }
        });

        Ok(())
    }

    fn on_message(&mut self, ws_msg: ws::Message) -> ws::Result<()> {
        debug!("on_message: {}", ws_msg);

        let msg: Message = val_or_send_msg_err!(ws_msg2msg(&ws_msg), self, "");

        match msg.kind.as_str() {
            "update" => {
                let tile_ids: Vec<String> = val_or_send_msg_err!(
                    serde_json::from_str(&msg.text),
                    self,
                    format!("FAILED unjsoning: '{}'", &msg.text)
                );
                let db = val_or_send_msg_err!(db::Db::new(), self, "Can't get db::Db");
                val_or_send_msg_err!(self.publish_tiles(tile_ids, &db), self, "Can't publish");
            }
            _ => {
                send_or_log_err(
                    &self.out,
                    &Message {
                        kind: String::from("error"),
                        text: format!("Unknown message kind: ({})", msg.kind),
                    },
                )
            }
        }

        Ok(())
    }

    fn on_close(&mut self, _code: ws::CloseCode, reason: &str) {
        debug!("on_close: {:?}", reason);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use db;
    use serde_json::Value;
    use test_utils;
    use utils;

    #[test]
    fn ws_sends_tile_data_when_it_is_requested() {
        // prepare data
        utils::load_config(None);
        let db = db::Db::new().unwrap();
        let dashboard_name = "dashboard-test";
        test_utils::upsert_dashboard(&db, &dashboard_name);
        db.upsert_tile(&dashboard_name, "tile-test", "{\"tile-data\": {}}")
            .unwrap();

        // websocket listener
        thread::spawn(move || {
            let ws_ip_port = from_config("DASHBOARD_WEBSOCKET_IP_PORT");
            run_ws_listener(&ws_ip_port);
        });
        // websocket client
        ws::connect(
            format!(
                "ws://{}/{}",
                from_config("DASHBOARD_WEBSOCKET_IP_PORT"),
                dashboard_name
            ),
            |out| {
                // client requests data from server
                let client_msg = Message {
                    kind: format!("update"),
                    text: format!("[\"tile-test\"]"),
                };
                out.send(serde_json::to_string(&client_msg).unwrap())
                    .unwrap();

                move |ws_msg| {
                    // receive update of tile data
                    let msg = ws_msg2msg(&ws_msg).unwrap();

                    assert_eq!(msg.kind, "tile");
                    assert_eq!(
                        serde_json::from_str::<Value>(&msg.text).unwrap(),
                        serde_json::from_str::<Value>(
                            "{\"tile-id\": \"tile-test\", \"tile-data\": {}}",
                        ).unwrap()
                    );
                    out.close(ws::CloseCode::Normal)
                }
            },
        ).unwrap()
    }

}
