extern crate paho_mqtt;

use std::{process, time};
use tokio;
use tokio_stream::StreamExt;

struct Args {
    pub mqtt_broker_uri: String,
    pub res_topic: String,
    pub req_topic: String,
    pub interval: u8,
    pub message: Option<String>,
    pub lifetime: u8,
}

impl Args {
    pub fn load_env() -> Self {
        let mqtt_broker_uri = {
            let broker_uri = std::env::var("HEALTHCHECK_MQTT_BROKER_URI");
            let variable_name = std::env::var("HEALTHCHECK_MQTT_BROKER_URI_VARIABLE_NAME");
            match (&broker_uri, &variable_name) {
                (Ok(_), _) => broker_uri,
                (Err(_), Ok(name)) => std::env::var(name),
                (Err(_), Err(_)) => broker_uri,
            }
        };
        if mqtt_broker_uri.is_err() {
            panic!("missing: neither HEALTHCHECK_MQTT_BROKER_URI nor HEALTHCHECK_MQTT_BROKER_URI_VARIABLE_NAME");
        }
        let service_name = std::env::var("SERVICE_NAME");
        let res_topic = {
            let topic = std::env::var("HEALTHCHECK_RES_TOPIC");
            match (&topic, &service_name) {
                (Ok(_), _) => topic,
                (Err(_), Ok(name)) => Ok(format!("{}/healthcheck_res", name)),
                (Err(_), Err(_)) => topic,
            }
        };
        let req_topic = {
            let topic = std::env::var("HEALTHCHECK_REQ_TOPIC");
            match (&topic, &service_name) {
                (Ok(_), _) => topic,
                (Err(_), Ok(name)) => Ok(format!("{}/healthcheck_req", name)),
                (Err(_), Err(_)) => topic,
            }
        };
        if res_topic.is_err() {
            panic!("missing: neither HEALTHCHECK_RES_TOPIC nor SERVICE_NAME");
        }
        if req_topic.is_err() {
            panic!("missing: neither HEALTHCHECK_REQ_TOPIC nor SERVICE_NAME");
        }
        let message = std::env::var("HEALTHCHECK_MESSAGE").ok();
        let interval = std::env::var("HEALTHCHECK_REQUEST_INTERVAL")
            .unwrap_or("2".into())
            .parse();
        if interval.is_err() {
            panic!("failed to parse HEALTHCHECK_REQUEST_INTERVAL, specify a value in [0, 255]");
        }
        let lifetime = std::env::var("HEALTHCHECK_REQUEST_LIFETIME")
            .unwrap_or("16".into())
            .parse();
        if lifetime.is_err() {
            panic!("failed to parse HEALTHCHECK_REQUEST_LIFETIME, specify a value in [0, 255]");
        }

        Self {
            mqtt_broker_uri: mqtt_broker_uri.unwrap(),
            res_topic: res_topic.unwrap(),
            req_topic: req_topic.unwrap(),
            message,
            interval: interval.unwrap(),
            lifetime: lifetime.unwrap(),
        }
    }
}

#[tokio::main]
async fn main() {
    let args = Args::load_env();

    let lifetime = args.lifetime;
    tokio::spawn(async move {
        tokio::time::sleep(time::Duration::from_secs(lifetime.into())).await;
        println!("lifetime has been exhausted");
        process::exit(1);
    });

    let mut client = {
        let suffix = time::SystemTime::now()
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let client_id = format!("mqtt-healthchecker_{:x}", suffix);
        let uri = args.mqtt_broker_uri.clone();
        let create_opts = paho_mqtt::CreateOptionsBuilder::new()
            .server_uri(&uri)
            .client_id(client_id)
            .finalize();
        let mut c = paho_mqtt::AsyncClient::new(create_opts).unwrap_or_else(|err| {
            panic!("Error creating the client: {:?}", err);
        });
        c.set_connected_callback(move |_| println!("MQTT Connection established: {}", uri));
        c
    };
    let mut rx = client.get_stream(64);
    if let Err(e) = client.connect(None).await {
        panic!("Unable to connect: {:?}", e)
    }
    match client.subscribe(&args.res_topic, 0).await {
        Ok(_) => println!("subscribed: {:?}", args.res_topic),
        Err(e) => {
            panic!("Error subscribes topics: {:?}", e);
        }
    }

    let req_topic = args.req_topic;
    let interval = args.interval;
    tokio::spawn(async move {
        let mut counter = 0u32;
        loop {
            let msg = paho_mqtt::Message::new(req_topic.as_str(), "healthcheck", 0);
            match client.publish(msg).await {
                Ok(_) => {
                    counter += 1;
                    println!("sent a request ({})", counter);
                }
                Err(e) => {
                    println!("failed to send a request: {:?}", e);
                }
            }
            tokio::time::sleep(time::Duration::from_secs(interval.into())).await
        }
    });

    while let Some(optional_message) = rx.next().await {
        let msg = match optional_message {
            Some(msg) => msg,
            None => {
                println!("Stream disruption");
                continue;
            }
        };
        let topic = msg.topic().to_string();
        let payload = std::str::from_utf8(msg.payload()).unwrap().to_string();
        println!("received: {} ({} bytes)", topic, payload.bytes().len());
        if args.res_topic != topic {
            continue;
        }
        match args.message {
            Some(ref msg) if *msg != payload => {
                println!("Unexpected payload: {:?}", payload);
                continue;
            }
            _ => {
                println!("sweet!");
                process::exit(0)
            }
        }
    }
}
