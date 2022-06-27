extern crate paho_mqtt;

use clap::{App, Arg, ArgMatches};
use std::{process, time};
use tokio;
use tokio_stream::StreamExt;

struct Args {
    pub host: String,
    pub res_topic: String,
    pub req_topic: String,
    pub interval: u8,
    pub timeout: u8,
    pub payload: String,
    pub expect: Option<String>,
}

impl From<ArgMatches> for Args {
    fn from(matches: ArgMatches) -> Self {
        let host = matches.value_of_t("host").unwrap();
        let req_topic = matches.value_of_t("request_topic").unwrap();
        let res_topic = matches.value_of_t("response_topic").unwrap();
        let interval = matches.value_of_t("interval").unwrap_or(2u8);
        let timeout = matches.value_of_t("timeout").unwrap_or(16u8);
        let payload = matches
            .value_of_t("payload")
            .unwrap_or("healthcheck".to_string());
        let expect = matches.value_of_t("expect").ok();
        Self {
            host,
            res_topic,
            req_topic,
            interval,
            timeout,
            payload,
            expect,
        }
    }
}

#[tokio::main]
async fn main() {
    let args: Args = App::new("mqtt-healthchecker-rs")
        .version("0.1.1")
        .author("Yusuke Sato <yusuke1.sato9.git@gmail.com>")
        .arg(
            Arg::new("host")
                .short('h')
                .long("host")
                .value_name("HOST")
                .help("Sets the MQTT broker host")
                .required(true),
        )
        .arg(
            Arg::new("request_topic")
                .long("request")
                .help("Sets the topic name to which sends requests")
                .value_name("REQUEST_TOPIC")
                .required(true),
        )
        .arg(
            Arg::new("response_topic")
                .long("response")
                .help("Sets the topic name to which the response is sent")
                .value_name("RESPONSE_TOPIC")
                .required(true),
        )
        .arg(
            Arg::new("payload")
                .short('p')
                .long("payload")
                .help("The payload for the healthckeck request (default: \"healthcheck\")"),
        )
        .arg(
            Arg::new("expect")
                .short('e')
                .long("expect")
                .help("The expected payload in the healthckeck response"),
        )
        .arg(
            Arg::new("interval")
                .short('i')
                .long("interval")
                .help("The interval period for sending a request (default: 2 seconds)"),
        )
        .arg(
            Arg::new("timeout")
                .short('t')
                .long("timeout")
                .help("The timeout (seconds) to exit with an error status (default: 16 seconds)"),
        )
        .get_matches()
        .into();

    let timeout = args.timeout;
    tokio::spawn(async move {
        tokio::time::sleep(time::Duration::from_secs(timeout.into())).await;
        println!("has reached the timeout: {}", timeout);
        process::exit(1);
    });

    let mut client = {
        let suffix = time::SystemTime::now()
            .duration_since(time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let client_id = format!("mqtt-healthchecker_{:x}", suffix);
        let uri = args.host.clone();
        let create_opts = paho_mqtt::CreateOptionsBuilder::new()
            .server_uri(&uri)
            .client_id(client_id)
            // ref. https://github.com/eclipse/paho.mqtt.rust/blob/v0.9.1/src/create_options.rs#L47-L68
            .persistence(None)
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
            let msg = paho_mqtt::Message::new(req_topic.as_str(), "healthcheck", 1);
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
        match args.expect {
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
