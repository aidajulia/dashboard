use redis;
use redis::Commands;
use serde_json;
use std::thread;
use utils::{from_config, redis_url};
use ws;

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

pub fn run_ws_listener(ip_port: String) {
    println!("Serving websocket on: {}", ip_port);
    ws::listen(ip_port.as_str(), |out| {
        Server {
            out: out,
            redis_url: redis_url(from_config("DASHBOARD_REDIS_IP_PORT").as_str()),
        }
    }).expect("starting websocket FAILED");
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    //TODO: replace String with &str?
    kind: String,
    text: String,
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
}

trait CanPublish {
    fn publish_tiles(&mut self, tile_ids: Vec<String>, redis: &redis::Client) -> ();
}

impl CanPublish for Server {
    fn publish_tiles(&mut self, tile_ids: Vec<String>, redis: &redis::Client) {
        for tile_id in &tile_ids {
            let result = match redis.get(tile_id) {
                Ok(json) => ("tile", json),
                Err(e) => (
                    "error",
                    format!("FAILED getting value for {} ({})", tile_id, e),
                ),
            };
            send_or_log_err(
                &self.out,
                &Message {
                    kind: String::from(result.0),
                    text: result.1,
                },
            )
        }
    }
}

impl ws::Handler for Server {
    fn on_request(&mut self, req: &ws::Request) -> ws::Result<(ws::Response)> {
        debug!("resource: {}", req.resource());
        ws::Response::from_request(req)
    }

    fn on_open(&mut self, _shake: ws::Handshake) -> ws::Result<()> {
        debug!("on_open");
        let redis_url = self.redis_url.clone();
        let channel_name = from_config("DASHBOARD_EVENTS_CHANNEL");
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
            loop {
                let msg = val_or_send_msg_err!(
                    pubsub.get_message(),
                    cloned_server,
                    "FAILED getting published message"
                );
                let payload: String = val_or_send_msg_err!(
                    msg.get_payload(),
                    cloned_server,
                    "FAILED getting payload of published \
                     message"
                );
                let json: String = val_or_send_msg_err!(
                    redis.get(&payload),
                    cloned_server,
                    format!("FAILED getting redis data for {}", &payload)
                );
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
                let redis = val_or_send_msg_err!(
                    redis::Client::open(self.redis_url.as_str()),
                    self,
                    "FAILED opening redis connection"
                );
                self.publish_tiles(tile_ids, &redis);
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
    use serde_json::Value;
    use utils::{get_redis_con, load_config};

    #[test]
    fn ws_sends_tile_data_when_it_is_requested() {
        load_config(None);

        // websocket listener
        thread::spawn(move || {
            let ws_ip_port = from_config("DASHBOARD_WEBSOCKET_IP_PORT");
            run_ws_listener(ws_ip_port);
        });

        // websocket client
        ws::connect(format!("ws://{}", from_config("DASHBOARD_WEBSOCKET_IP_PORT")),
                    |out| {
            // set tile data in redis
            let con = get_redis_con(from_config("DASHBOARD_REDIS_IP_PORT").as_str()).unwrap();
            let tile_data: Value =
                serde_json::from_str("{\"tile-id\": \"tile_id\", \"tile-data\": {}}").unwrap();
            con.set::<_, _, ()>("tile_id", serde_json::to_string(&tile_data).unwrap())
                .unwrap();

            // request update of tile data
            let msg = Message {
                kind: format!("update"),
                text: format!("[\"tile_id\"]"),
            };
            out.send(serde_json::to_string(&msg).unwrap()).unwrap();
            move |ws_msg| {
                // receive update of tile data
                let msg = ws_msg2msg(&ws_msg).unwrap();
                let text: Value = serde_json::from_str(msg.text.as_str()).unwrap();
                assert_eq!(msg.kind, "tile");
                assert_eq!(text, tile_data);
                out.close(ws::CloseCode::Normal)
            }
        })
            .unwrap()
    }

}
