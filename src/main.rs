use rumqttc;
use serde_json;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

mod matrix;

const UNIQUE_ID: &str = "sams_led_matrix";

fn create_mqtt_client() -> (rumqttc::Client, rumqttc::Connection) {
    let mut options = rumqttc::MqttOptions::new(UNIQUE_ID, "homeassistant.local", 1883);
    options.set_keep_alive(Duration::from_secs(5));
    options.set_credentials("mqtt", "mqtt");
    rumqttc::Client::new(options, 10)
}

fn main() {
    let (mut client, mut connection) = create_mqtt_client();
    let topic = format!("homeassistant/select/{}", UNIQUE_ID);

    let config = serde_json::json!({
        "name": "LED Matrix",
        "command_topic": format!("{}/set", topic),
        "options": ["blank","colourcycle","onair"],
        "retain": true,
        "unique_id": UNIQUE_ID,
    });

    client
        .publish(
            format!("{}/config", topic),
            rumqttc::QoS::AtLeastOnce,
            false,
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();

    client
        .subscribe(format!("{}/#", topic), rumqttc::QoS::AtMostOnce)
        .unwrap();

    let (tx, rx) = channel();

    thread::spawn(move || {
        matrix::run(rx);
    });

    for notification in connection.iter() {
        match notification {
            Ok(event) => match event {
                rumqttc::Event::Incoming(packet) => match packet {
                    rumqttc::Packet::Publish(publish) => {
                        let parts = publish.topic.split("/").collect::<Vec<_>>();
                        let command = parts[3];

                        let payload = String::from_utf8(publish.payload.to_vec()).unwrap();

                        match command.to_lowercase().as_str() {
                            "set" => tx.send(payload).unwrap(),
                            _ => (),
                        }
                    }
                    _ => (),
                },
                _ => (),
            },
            Err(error) => println!("Error receiving notification: {:?}", error),
        }
    }
}
