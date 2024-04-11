//! This `hub` crate is the
//! entry point of the Rust logic.

use std::net::{Ipv4Addr, SocketAddr};

// This `tokio` will be used by Rinf.
// You can replace it with the original `tokio`
// if you're not targeting the web.
use tokio_with_wasm::tokio;

mod messages;
mod sample_functions;
use messages::chatmessage::*;
use tokio::net::UdpSocket;

rinf::write_interface!();

async fn main() {
    
    let forward_ip_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 8001);
    let self_send_addr = "127.0.0.1:8004"; //2 4
    let self_recv_addr = "127.0.0.1:8005"; //3 5
    let backward_ip_addr = SocketAddr::new(Ipv4Addr::new(127, 0, 0, 1).into(), 8003); //5 3

    let mut buf = [0; 1024];

    tokio::spawn(async move {
        let socket = UdpSocket::bind(self_recv_addr).await.unwrap();
        loop {
            let (len, recv_addr) = socket.recv_from(&mut buf).await.unwrap();
            let msg = String::from_utf8_lossy(&buf[..len]);

            if forward_ip_addr == recv_addr {
                socket.send_to(msg.as_bytes(), &backward_ip_addr).await.unwrap();
            }
            else if backward_ip_addr == recv_addr {
                socket.send_to(msg.as_bytes(), &forward_ip_addr).await.unwrap();
            } else {
                // this is newbie
                socket.send_to(msg.as_bytes(), &recv_addr).await.unwrap();
            }

            println!("Received: {} from {}", msg, recv_addr);

            RecvMessage{
                who: recv_addr.to_string(),
                contents: msg.to_string(),
            }.send_signal_to_dart(None);
        }
    });


    // let socket = UdpSocket::bind("127.0.0.1:0").await.unwrap();
    // socket.connect("127.0.0.1:8080").await.unwrap();

    let mut MessageSender: tokio::sync::mpsc::Receiver<rinf::DartSignal<SendMessage>> = SendMessage::get_dart_signal_receiver();
    tokio::spawn(async move {
        let socket = UdpSocket::bind(self_send_addr).await.unwrap();
        while let Some(dart_signal) = MessageSender.recv().await {
            let msg = dart_signal.message;
            let send_msg = msg.contents;
            
            println!("will send: {}", send_msg);
            // socket.send(send_msg.as_bytes()).await.unwrap();
            socket.send_to(send_msg.as_bytes(), &forward_ip_addr).await.unwrap();
            socket.send_to(send_msg.as_bytes(), &backward_ip_addr).await.unwrap();
            println!("Client sent: {}", send_msg);
        }
    });

}
