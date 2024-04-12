//! This `hub` crate is the
//! entry point of the Rust logic.

use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, ptr::null, thread::sleep, time::Duration};
// use std::sync::{Arc, Mutex};
use tokio::sync::Mutex;
use std::sync::Arc;
// This `tokio` will be used by Rinf.
// You can replace it with the original `tokio`
// if you're not targeting the web.
use tokio_with_wasm::tokio;
use tokio::sync::RwLock;

mod messages;
mod sample_functions;
use messages::chatmessage::*;
use tokio::net::UdpSocket;

rinf::write_interface!();

async fn main() {

    tokio::spawn(udp_machine());

}

#[derive(Clone)]
struct SharedState {
    forward_ip_addr: SocketAddr,
    backward_ip_addr: SocketAddr,
}

pub async fn udp_machine(){

    let shared_state = Arc::new(Mutex::new(SharedState {
        forward_ip_addr: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 8001),
        backward_ip_addr: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), 8001),
    }));

    let a_shared_state_knocker = Arc::clone(&shared_state);
    let a_shared_state_sender = Arc::clone(&shared_state);
    let a_shared_state_reciver = Arc::clone(&shared_state);

    // let forward_ip_addr = SocketAddr::new(forward_ipv4.into(), 8001);
    let self_send_addr = "0.0.0.0:0";
    let self_recv_addr = "127.0.0.1:2223";
    // let backward_ip_addr = SocketAddr::new(backward_ipv4.into(), 8003);


    tokio::spawn(async move {
        let socket = UdpSocket::bind(self_recv_addr).await.unwrap();
        loop {
            let mut buf = [0; 1024];
            let (len, recv_addr) = socket.recv_from(&mut buf).await.unwrap();
            let msg = String::from_utf8_lossy(&buf[..len]);

            let mut state = a_shared_state_reciver.lock().await;
            // let mut state = shared_state.clone();
            
            RecvMessage{
                who: recv_addr.ip().to_string(),
                contents: msg.to_string(),
            }.send_signal_to_dart(None);
            
            if state.forward_ip_addr == recv_addr {
                if state.backward_ip_addr.ip() != Ipv4Addr::new(0, 0, 0, 0) {
                    let _ = socket.send_to(msg.as_bytes(), &state.backward_ip_addr).await;
                }
            }
            else if state.backward_ip_addr == recv_addr {
                if state.forward_ip_addr.ip() != Ipv4Addr::new(0, 0, 0, 0) {
                    let _ = socket.send_to(msg.as_bytes(), &state.forward_ip_addr).await;
                }
            } else {
                // this is newbie

                if state.forward_ip_addr.ip() == Ipv4Addr::new(0, 0, 0, 0) {
                    state.forward_ip_addr = SocketAddr::new(recv_addr.ip().into(), 2222);
                }
                else if state.backward_ip_addr.ip() == Ipv4Addr::new(0, 0, 0, 0) {
                    state.backward_ip_addr = SocketAddr::new(recv_addr.ip().into(), 2224);
                } else {
                    // this is real newbie
                    let _ = socket.send_to(msg.as_bytes(), &recv_addr).await;
                }
            }
            
            println!("Received: {} from {}", msg, recv_addr);
            sleep(Duration::from_millis(15));
            RecvMessage{
                who: "".to_string(),
                contents: "".to_string(),
            }.send_signal_to_dart(None);
        }
    });

    let mut _message_sender: tokio::sync::mpsc::Receiver<rinf::DartSignal<SendMessage>> = SendMessage::get_dart_signal_receiver();
    tokio::spawn(async move {
        let socket = UdpSocket::bind(self_send_addr).await.unwrap();
        while let Some(dart_signal) = _message_sender.recv().await {
            let msg = dart_signal.message;
            let send_msg = msg.contents;

            let state = a_shared_state_sender.lock().await;
            
            if (state.forward_ip_addr.ip() != Ipv4Addr::new(0, 0, 0, 0)) {
                socket.send_to(send_msg.as_bytes(), &state.forward_ip_addr).await;
            }
            if (state.backward_ip_addr.ip() != Ipv4Addr::new(0, 0, 0, 0)) {
                socket.send_to(send_msg.as_bytes(), &state.backward_ip_addr).await;
            }
            println!("Client sent: {}", send_msg);
        }
    });

    let mut _enter_new_ip: tokio::sync::mpsc::Receiver<rinf::DartSignal<KnockIp>> = KnockIp::get_dart_signal_receiver();
    tokio::spawn(async move {
        let socket = UdpSocket::bind(self_send_addr).await.unwrap();
        while let Some(dart_signal) = _enter_new_ip.recv().await {
            let msg = dart_signal.message;

            let handshakemsg = "HIHI";

            let newIP = SocketAddr::new(
                Ipv4Addr::new(msg.ip_addr_int1 as u8,
                    msg.ip_addr_int2 as u8,
                    msg.ip_addr_int3 as u8,
                    msg.ip_addr_int4 as u8).into(), msg.port as u16);

            // let state = a_shared_state_knocker.lock().await;
            socket.send_to(handshakemsg.as_bytes(), &newIP).await;
            
            println!("Client sent: {}", handshakemsg);
        }
    });

}
