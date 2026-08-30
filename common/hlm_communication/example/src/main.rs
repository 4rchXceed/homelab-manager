use hlm_communication::{
    builders::{
        message_builder::ProtocolMessage,
        message_parts::{
            binary::BinaryPart,
            parsable_part::{ParsableMessage, ParsedMessageParts, ProtocolMessageData},
            string::StringPart,
        },
        protocol_builder::ProtocolBuilder,
        send::message::MessageBuilder,
        server_socket::ServerSocket,
    },
    config::SSLConfig,
};
use serde_json::{Value, json};

use crate::custom_part::JsonPart;

mod custom_part;

struct PrintMessageData {
    message: String,
}

impl ProtocolMessageData for PrintMessageData {
    fn build() -> ParsedMessageParts {
        return ParsedMessageParts::new().add_part("message", StringPart::until('\n'));
    }
    fn when_parsed(parts: &ParsedMessageParts) -> Option<Self> {
        let value = parts.get("message")?;
        let str = value.as_str()?;
        return Some(Self {
            message: String::from(str),
        });
    }
}

struct SuccessData {
    success: bool,
}

impl ProtocolMessageData for SuccessData {
    fn build() -> ParsedMessageParts {
        return ParsedMessageParts::new().add_part("success", BinaryPart::size(1));
    }
    fn when_parsed(parts: &ParsedMessageParts) -> Option<Self> {
        let success_bytes = parts.get("success")?;

        let success_bytes = success_bytes.as_bytes()?;

        return Some(Self {
            success: success_bytes[0] == 0x01,
        });
    }
}

struct ExampleJsonData {
    json: Value,
}

impl ProtocolMessageData for ExampleJsonData {
    fn build() -> ParsedMessageParts {
        return ParsedMessageParts::new().add_part("json_datas", JsonPart::create());
    }
    fn when_parsed(parts: &ParsedMessageParts) -> Option<Self> {
        let json = parts.get("json_datas")?.into::<Value>();
        if json.is_some() {
            return Some(Self {
                json: json.unwrap(),
            });
        } else {
            return None;
        }
    }
}

struct ComplexOneData {
    message: String,
    success: bool,
    id: u32,
}

impl ProtocolMessageData for ComplexOneData {
    fn build() -> ParsedMessageParts {
        return ParsedMessageParts::new()
            .add_part("message", StringPart::until('\n'))
            .add_part("success", BinaryPart::size(1))
            .add_part("id", BinaryPart::size(4));
    }
    fn when_parsed(parts: &ParsedMessageParts) -> Option<Self> {
        let message = parts.get("message")?;
        let message = message.as_str()?;
        let success = parts.get("success")?.as_bytes()?[0] == 0x01;
        let id_bytes = parts.get("id")?;
        let id_bytes = id_bytes.as_bytes()?;
        if id_bytes.len() != 4 {
            return None;
        }
        let id = u32::from_be_bytes([id_bytes[0], id_bytes[1], id_bytes[2], id_bytes[3]]);
        return Some(Self {
            message: String::from(message),
            success: success,
            id: id,
        });
    }
}

fn print_hello_world(_: ParsableMessage) -> Option<Vec<u8>> {
    println!("Hello World!");
    return None;
}

fn print_message_and_success(msg: ParsableMessage) -> Option<Vec<u8>> {
    let message = msg.parse::<PrintMessageData>();
    if message.is_ok() {
        println!("{}", message.unwrap().message);
    } else {
        println!("INVALID message");
    }
    return MessageBuilder::new(BinaryPart::new(&[0x03]))
        .add_part(BinaryPart::new(&[0x01]))
        .build();
}

fn panic_if_error(msg: ParsableMessage) -> Option<Vec<u8>> {
    let success = msg.parse::<SuccessData>();
    if success.is_err() || !success.as_ref().unwrap().success {
        panic!("Not success: {:?}", success.err());
    } else {
        println!("Success!");
        return None;
    }
}

fn example_json_custom_part(msg: ParsableMessage) -> Option<Vec<u8>> {
    let json = msg.parse::<ExampleJsonData>();
    if json.is_ok() {
        println!("JSON: {}", json.unwrap().json);
    } else {
        println!("INVALID JSON: {}", json.err().unwrap());
    }
    return None;
}

fn complex_one(msg: ParsableMessage) -> Option<Vec<u8>> {
    let data = msg.parse::<ComplexOneData>();
    if data.is_ok() {
        let data = data.unwrap();
        println!(
            "Message: {}, Success: {}, ID: {}",
            data.message, data.success, data.id
        );
    } else {
        println!("INVALID ComplexOneData {}", data.err().unwrap());
    }
    return MessageBuilder::new(BinaryPart::new(&[0x03]))
        .add_part(BinaryPart::new(&[0x01]))
        .build();
}

fn register_types(message: ProtocolMessage) -> ProtocolMessage {
    return message
        .for_type(&[0x01], print_hello_world)
        .for_type(&[0x02], print_message_and_success)
        .for_type(&[0x03], panic_if_error)
        .for_type(&[0x04], example_json_custom_part)
        .for_type(&[0x05], complex_one);
}

fn get_message() -> ProtocolMessage {
    let message = ProtocolMessage::new()
        .add_type_part(BinaryPart::size(1))
        .with_end_message(BinaryPart::new(&[0x00]));
    let message = register_types(message);
    return message;
}

fn ensure_cert() {
    if !std::path::Path::new("./cert.crt").exists() {
        println!(
            "Certificate file not found: ./cert.crt... generating a server certificate for testing purposes"
        );
        let res = SSLConfig::new_generated_server(
            String::from("cert.crt"),
            String::from("key.key"),
            String::from("ca.crt"),
            vec![String::from("127.0.0.1")],
        );
        if res.is_err() {
            panic!(
                "Failed to generate server certificate: {:?}",
                res.err().unwrap()
            );
        }
    }
}

fn client() -> ProtocolBuilder {
    ensure_cert();
    return ProtocolBuilder::client()
        .on_error(|e| {
            panic!("Error [Client]: {:?}", e);
        })
        .with_ssl_config(SSLConfig::new_client(String::from("ca.crt")))
        .with_protocol_message(get_message);
}

fn handle_client(socket: ServerSocket) -> bool {
    let ip = socket.ip();
    if ip.is_some() {
        println!("Client connected: {}", ip.unwrap());
    } else {
        println!("Client connected: Unknown IP");
    }
    std::thread::spawn(move || {
        handle_client_threaded(socket);
    });
    return false;
}

fn handle_client_threaded(mut socket: ServerSocket) {
    socket
        .only_send(
            MessageBuilder::new(BinaryPart::new(&[0x01]))
                .build()
                .unwrap(),
        )
        .expect("Failed to send message");
    socket
        .send_pingpong(
            MessageBuilder::new(BinaryPart::new(&[0x02]))
                .add_part(StringPart::new(
                    "Hello, this message was send from the server.",
                    '\n',
                ))
                .build()
                .unwrap(),
        )
        .expect("Failed to send message");
    socket
        .only_send(
            MessageBuilder::new(BinaryPart::new(&[0x04]))
                .add_part(JsonPart::new(json!({
                    "test": true,
                    "test2": [
                        {
                            "test3": "test4"
                        }
                    ]
                })))
                .build()
                .unwrap(),
        )
        .expect("Failed to send message");
    socket
        .send_pingpong(
            MessageBuilder::new(BinaryPart::new(&[0x05]))
                .add_part(StringPart::new("Test datas", '\n'))
                .add_part(BinaryPart::new(&[0x01]))
                .add_part(BinaryPart::new(&3939u32.to_be_bytes()))
                .build()
                .unwrap(),
        )
        .expect("Failed to send message");

    loop {
        let mut buf = String::new();
        std::io::stdin()
            .read_line(&mut buf)
            .expect("Failed to read line");
        let buf = buf.trim();
        if buf == "exit" {
            println!("Exiting...");
            socket.end().unwrap();
            break;
        }
        socket
            .send_pingpong(
                MessageBuilder::new(BinaryPart::new(&[0x02]))
                    .add_part(StringPart::new(buf, '\n'))
                    .build()
                    .unwrap(),
            )
            .unwrap();
    }
}
fn server() -> ProtocolBuilder {
    ensure_cert();
    return ProtocolBuilder::server()
        .on_error(|e| {
            panic!("Error [Server]: {:?}", e);
        })
        .on_listen(|| {
            println!("Server is listening...");
        })
        .with_server_accept(handle_client)
        .with_ssl_config(SSLConfig::new_server(
            String::from("key.key"),
            String::from("cert.crt"),
        ))
        .with_protocol_message(get_message);
}

fn main() {
    let side = std::env::args().nth(1);
    if side.is_none() {
        println!("Please specify a side: client or server");
        return;
    }
    let side = side.unwrap();
    if side == "client" {
        let mut client = client();
        client
            .connect("127.0.0.1", 12345)
            .expect("Failed to connect to server");
    } else if side == "server" {
        let mut server = server();
        server
            .listen(12345, Some("127.0.0.1"))
            .expect("Failed to listen on server");
    } else {
        println!("Invalid side: {}", side);
    }
}
