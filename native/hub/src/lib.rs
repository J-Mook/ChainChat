//! This `hub` crate is the
//! entry point of the Rust logic.

use std::{net::{IpAddr, Ipv4Addr, SocketAddr}, ptr::null, thread::sleep, time::Duration};
use rand::Rng;

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

use local_ip_address::local_ip;

rinf::write_interface!();

async fn main() {

    tokio::spawn(udp_machine());

}

#[derive(Clone)]
struct SharedState {
    forward_ip_addr: SocketAddr,
    backward_ip_addr: SocketAddr,
}

const RECV_PORT:u16 = 6113;
const SELF_RECV_PORT:u16 = 6112;

pub async fn udp_machine(){

    let shared_state = Arc::new(Mutex::new(SharedState {
        forward_ip_addr: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), RECV_PORT),
        backward_ip_addr: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), RECV_PORT),
    }));

    let a_shared_state_knocker = Arc::clone(&shared_state);
    let a_shared_state_sender = Arc::clone(&shared_state);
    let a_shared_state_reciver = Arc::clone(&shared_state);

    let self_send_addr = "0.0.0.0:0";
    let self_recv_addr = SocketAddr::new(local_ip().unwrap().into(), SELF_RECV_PORT);


    tokio::spawn(async move {
        let socket = UdpSocket::bind(self_recv_addr).await.unwrap();
        loop {
            let mut buf = [0; 1024];
            let (len, recv_addr) = socket.recv_from(&mut buf).await.unwrap();
            let msg = String::from_utf8_lossy(&buf[..len]);

            let mut state = a_shared_state_reciver.lock().await;
            
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
                    state.forward_ip_addr = SocketAddr::new(recv_addr.ip().into(), RECV_PORT);
                }
                else if state.backward_ip_addr.ip() == Ipv4Addr::new(0, 0, 0, 0) {
                    state.backward_ip_addr = SocketAddr::new(recv_addr.ip().into(), RECV_PORT);
                } else {
                    // this is real newbie
                    let _ = socket.send_to(msg.as_bytes(), &recv_addr).await;
                }
            }
            
            println!("Received: {} from {}", msg, recv_addr);
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
    let mut _password_generate: tokio::sync::mpsc::Receiver<rinf::DartSignal<GetMyPassword>> = GetMyPassword::get_dart_signal_receiver();
    tokio::spawn(async move {
        while let Some(dart_signal) = _password_generate.recv().await {

            if let SocketAddr::V4(v4_addr) = self_recv_addr {
                let ip = v4_addr.ip();
                let port = v4_addr.port();
                let octets = ip.octets();
        
                let mut transformed = String::new();
                let rand_num1 = rand::thread_rng().gen_range(0..=25);
                let rand_num2 = rand::thread_rng().gen_range(0..=25);
                transformed.push((rand_num1 as u8 + 65) as char);
                transformed.push((rand_num2 as u8 + 65) as char);
                transformed.push((((octets[0] / 26 + rand_num1) % 26) as u8 + 65) as char);
                transformed.push((((octets[0] % 26 + rand_num2) % 26) as u8 + 65) as char);
                transformed.push((((octets[1] / 26 + rand_num1) % 26) as u8 + 65) as char);
                transformed.push((((octets[1] % 26 + rand_num2) % 26) as u8 + 65) as char);
                transformed.push((((octets[2] / 26 + rand_num1) % 26) as u8 + 65) as char);
                transformed.push((((octets[2] % 26 + rand_num2) % 26) as u8 + 65) as char);
                transformed.push((((octets[3] / 26 + rand_num1) % 26) as u8 + 65) as char);
                transformed.push((((octets[3] % 26 + rand_num2) % 26) as u8 + 65) as char);
                println!("I'm {}.{}.{}.{} : {}", octets[0], octets[1], octets[2], octets[3], transformed);
                
                ThisisMyPassword{
                    password: transformed,
                }.send_signal_to_dart(None);
            }
        }
    });

    let mut _enter_new_ip: tokio::sync::mpsc::Receiver<rinf::DartSignal<KnockIp>> = KnockIp::get_dart_signal_receiver();
    tokio::spawn(async move {
        let socket = UdpSocket::bind(self_send_addr).await.unwrap();
        while let Some(dart_signal) = _enter_new_ip.recv().await {
            let msg = dart_signal.message;

            let handshakemsg = "HIHI";
            let passowrd = msg.password;
            println!("{} ", passowrd);

            let key_num1 = passowrd.chars().nth(0).unwrap() as u16;
            let key_num2 = passowrd.chars().nth(1).unwrap() as u16;
            let oct11 = ((passowrd.chars().nth(2).unwrap() as u16 + 26) - key_num1) % 26;
            let oct12 = ((passowrd.chars().nth(3).unwrap() as u16 + 26) - key_num2) % 26;
            let oct21 = ((passowrd.chars().nth(4).unwrap() as u16 + 26) - key_num1) % 26;
            let oct22 = ((passowrd.chars().nth(5).unwrap() as u16 + 26) - key_num2) % 26;
            let oct31 = ((passowrd.chars().nth(6).unwrap() as u16 + 26) - key_num1) % 26;
            let oct32 = ((passowrd.chars().nth(7).unwrap() as u16 + 26) - key_num2) % 26;
            let oct41 = ((passowrd.chars().nth(8).unwrap() as u16 + 26) - key_num1) % 26;
            let oct42 = ((passowrd.chars().nth(9).unwrap() as u16 + 26) - key_num2) % 26;
            
            let oct1 = oct11 * 26 + oct12;
            let oct2 = oct21 * 26 + oct22;
            let oct3 = oct31 * 26 + oct32;
            let oct4 = oct41 * 26 + oct42;
            
            println!("{} Solve : {}.{}.{}.{}", passowrd, oct1, oct2, oct3, oct4);

            let newIP = SocketAddr::new(Ipv4Addr::new(oct1 as u8,oct2 as u8,oct3 as u8,oct4 as u8).into(), RECV_PORT as u16);

            socket.send_to(handshakemsg.as_bytes(), &newIP).await;

            println!("Client sent: {}", handshakemsg);
        }
    });

}
