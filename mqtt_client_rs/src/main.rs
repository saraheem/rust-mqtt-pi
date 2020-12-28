use chrono::prelude::*;
use const_format::formatcp;
use rumqttc::{AsyncClient, Event, Incoming, LastWill, MqttOptions, Outgoing, QoS, SubscribeTopic};
use serde_json::Value;
use std::str::from_utf8;

const CLIENT_ID: &str = "WKBXXB0054";
struct Topic {}
impl Topic {
    const PRESENCE: &'static str = "bw3/presence";
    const PING: &'static str = formatcp!("bw3/4/{}/ping", CLIENT_ID);
    const PONG: &'static str = formatcp!("bw3/4/{}/pong", CLIENT_ID);
    const EVM: &'static str = formatcp!("bw3/4/{}/evm3", CLIENT_ID);
    const IOT: &'static str = formatcp!("bw3/4/{}/iot", CLIENT_ID);
}

struct IotRspTemplate {}
impl IotRspTemplate {
    fn get_rsp_payload(conv_id: &str) -> (String, String) {
        // acknowledgment template response
        let ack = format!(
            r#"{{"ConversationID":"{}","MessageType":"Ack","MessageOrigin":null,"From":"{}","CommandInvokerGuid":null,"To":"admin","TimestampOrigin":1607499037857,"TimestampHub":1607499037857,"Body":"{{\r\n \"Code\": 200,\r\n \"Description\": \"Message Received\",\r\n \"OriginalMessage\": {{\r\n \"ConversationID\": \"23439f1c-98f1-4bbd-8e22-e3f86055e6df\",\r\n \"MessageType\": \"Command\",\r\n \"MessageOrigin\": null,\r\n \"From\": \"admin\",\r\n \"CommandInvokerGuid\": \"012bf825-73b5-4eea-afa5-94e41ae157de\",\r\n \"To\": \"WKBXXB0054\",\r\n \"TimestampOrigin\": 1607499037775,\r\n \"TimestampHub\": 1607499037776,\r\n \"Body\": \"{{\\\"CommandType\\\":\\\"InCarStatus\\\",\\\"CommandID\\\":10501,\\\"Params\\\":{{\\\"Duration\\\":60}}}}\",\r\n \"Data\": null\r\n }}\r\n}}","Data":null}}"#,
            conv_id, CLIENT_ID
        );

        // success template response
        let succ = format!(
            r#"{{"ConversationID":"{}","MessageType":"Success","MessageOrigin":null,"From":"{}","CommandInvokerGuid":"012bf825-73b5-4eea-afa5-94e41ae157de","To":"admin","TimestampOrigin":1607499037775,"TimestampHub":1607499037776,"Body":"{{\"success\": true,\"Result\": \"{{\\\"success\\\":true,\\\"Data\\\":{{\\\"status\\\":\\\"StreamingOff\\\",\\\"commandInvoker\\\":\\\"\\\"}}}}\",\"CommandType\": \"InCarStatus\",\"CommandID\": 10501,\"Description\": \"Operation succeeded.\"}}","Data":null}}"#,
            conv_id, CLIENT_ID
        );

        (ack, succ)
    }
}

async fn handle_on_connect(mut client: &mut AsyncClient, packet: rumqttc::ConnAck) {
    async fn listen_on(client: &mut AsyncClient) {
        // client to listen on the following topics
        let topic_ping = SubscribeTopic::new(Topic::PING.to_string(), QoS::AtMostOnce);
        let topic_evm = SubscribeTopic::new(Topic::EVM.to_string(), QoS::AtMostOnce);

        // subscribe to said topics
        client
            .subscribe_many(vec![topic_ping, topic_evm])
            .await
            .unwrap();
    }
    println!(
        "Connection Established. Connection Code : {:?}\nPublishing 'ONLINE' Presence\n",
        packet.code
    );

    // publish online presence on successful connection
    let presence = format!(r#"{{"ClientId":"{}","StationId":4,"Status":1}}"#, CLIENT_ID);
    client
        .publish(
            Topic::PRESENCE.to_string(),
            QoS::AtMostOnce,
            false,
            presence,
        )
        .await
        .unwrap();

    // handle client subscription for relevant topics
    listen_on(&mut client).await;
}

async fn handle_evm_msg(client: &mut AsyncClient, packet: rumqttc::Publish) {
    // match incoming message topic from EVM i.e. /../ping or /../evm3
    match packet.topic.as_str() {
        Topic::PING => {
            // extract payload
            let msg_payload = from_utf8(&packet.payload[..]).unwrap();

            //debug build : log to console
            println!("\nTopic : {}, Payload {:?}", packet.topic, packet.payload);

            // send payload on /../pong topic
            client
                .publish(Topic::PONG.to_string(), QoS::AtMostOnce, false, msg_payload)
                .await
                .unwrap();
        }
        Topic::EVM => {
            // parse message payload into JSON
            let msg_payload = from_utf8(&packet.payload[..]).unwrap();
            let payload_json: Value = serde_json::from_str(msg_payload).unwrap();

            // extract 'ConversationID
            let conv_id = payload_json["ConversationID"].as_str().unwrap();

            //debug build : log to console
            println!("\nTopic : {}, Payload : {:?}", packet.topic, packet.payload);

            // embed 'ConversationID' into each template responses
            // for 'acknowledgment' and 'success'

            let (ack, succ) = IotRspTemplate::get_rsp_payload(conv_id);
            client
                .publish(Topic::IOT.to_string(), QoS::AtMostOnce, false, ack)
                .await
                .unwrap();

            client
                .publish(Topic::IOT.to_string(), QoS::AtMostOnce, false, succ)
                .await
                .unwrap();
        }
        // make irrefutable pattern - ignore all other topics
        &_ => (),
    }
}

#[tokio::main]
async fn main() {
    let keep_alive_interval = 60 * 5;

    // last will message - publish offline presence
    let lwt = LastWill::new(
        Topic::PRESENCE.to_string(),
        QoS::AtMostOnce,
        format!(r#"{{"ClientId":"{}","StationId":4,"Status":0}}"#, CLIENT_ID),
    );

    let mut mqttoptions = MqttOptions::new(CLIENT_ID, "broker.mqttdashboard.com", 1883);
    mqttoptions.set_keep_alive(keep_alive_interval);
    mqttoptions.set_last_will(lwt);

    let (mut client, mut eventloop) = AsyncClient::new(mqttoptions, 20);

    loop {
        let dt = Local::now();
        if let Ok(event) = eventloop.poll().await {
            match event {
                Event::Incoming(Incoming::Publish(p)) => {
                    println!("\n[{}:{}:{}]", dt.hour(), dt.minute(), dt.second());
                    // handler for incoming messages
                    handle_evm_msg(&mut client, p).await;
                }
                Event::Incoming(Incoming::ConnAck(p)) => {
                    println!("\n[{}:{}:{}]", dt.hour(), dt.minute(), dt.second());
                    // handler for successful connection
                    handle_on_connect(&mut client, p).await;
                }
                // event does not fire
                Event::Outgoing(Outgoing::Disconnect) => {
                    println!("\nDisconnected from Broker");
                }
                // event fires within keep alive interval
                Event::Outgoing(Outgoing::PingReq) => {
                    println!("\n[{}:{}:{}]", dt.hour(), dt.minute(), dt.second());
                    println!("Pinging Broker");
                }
                // event fires if broker is alive after ping request from client
                Event::Incoming(Incoming::PingResp) => {
                    println!("Broker Alive");
                }
                _ => (),
            }
        // debug build : Log error message 'ConnectionError' to console
        // more information on : https://docs.rs/rumqttc/0.3.0/rumqttc/enum.ConnectionError.html
        } else if let Err(e) = eventloop.poll().await {
            println!("\n[{}:{}:{}]", dt.hour(), dt.minute(), dt.second());
            println!("Connection Error : {}", e);
        }
    }
}
