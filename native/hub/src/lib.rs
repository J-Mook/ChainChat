//! This `hub` crate is the
//! entry point of the Rust logic.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use rand::Rng;

use tokio::sync::Mutex;
use std::sync::Arc;
// This `tokio` will be used by Rinf.
// You can replace it with the original `tokio`
// if you're not targeting the web.
use tokio_with_wasm::tokio;

mod messages;
mod sample_functions;
use messages::chatmessage::*;
use tokio::net::UdpSocket;

use local_ip_address::local_ip;

rinf::write_interface!();

#[derive(Clone)]
struct SharedState {
    forward_ip_addr: SocketAddr,
    backward_ip_addr: SocketAddr,
}

const RECV_PORT:u16 = 6113;
const SELF_RECV_PORT:u16 = 6112;

async fn main() {

    let shared_state = Arc::new(Mutex::new(SharedState {
        forward_ip_addr: SocketAddr::new(Ipv4Addr::new(192,168,0,9).into(), RECV_PORT),
        backward_ip_addr: SocketAddr::new(Ipv4Addr::new(0, 0, 0, 0).into(), RECV_PORT),
    }));

    let a_shared_state_knocker = Arc::clone(&shared_state);
    let a_shared_state_sender = Arc::clone(&shared_state);
    let a_shared_state_reciver = Arc::clone(&shared_state);

    let local_ip = local_ip().unwrap();
    let self_send_addr = "0.0.0.0:0";
    let self_recv_addr = SocketAddr::new(local_ip.into(), SELF_RECV_PORT);


    tokio::spawn(async move {
        let socket = UdpSocket::bind(self_recv_addr).await.unwrap();
        loop {
            let mut buf = [0; 1024];
            let (len, recv_addr) = socket.recv_from(&mut buf).await.unwrap();
            let msg = String::from_utf8_lossy(&buf[..len]);

            let mut state = a_shared_state_reciver.lock().await;
            
            
            let msg_content = msg.to_string();
            if msg_content.chars().nth(0) != Some('\\') {
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
                    // // this is newbie
                    // if state.forward_ip_addr.ip() == Ipv4Addr::new(0, 0, 0, 0) {
                    //     state.forward_ip_addr = SocketAddr::new(recv_addr.ip().into(), RECV_PORT);
                    // }
                    // else if state.backward_ip_addr.ip() == Ipv4Addr::new(0, 0, 0, 0) {
                    //     state.backward_ip_addr = SocketAddr::new(recv_addr.ip().into(), RECV_PORT);
                    // } else {
                    //     // this is real newbie
                    //     let _ = socket.send_to(msg.as_bytes(), &recv_addr).await;
                    // }
                }
            } else {
                if let Some(idx) = msg_content.find("\\NiceToMeetYou"){
                    if idx == 0 {
                        let (oct1, oct2, oct3, oct4, port) = decryption(&msg_content[14..].to_string());
                        state.backward_ip_addr = SocketAddr::new(Ipv4Addr::new(oct1,oct2,oct3,oct4).into(), port);
                        
                        let ret = socket.send_to(format!("\\SetForwardIP{}", encryptionIP(self_recv_addr)).as_bytes(), &recv_addr).await; // SetForwardIP selfIP
                        match ret { Ok(_) => println!(" (Ok)"), Err(_) => println!(" (Fail)") };
                        let ret = socket.send_to(format!("\\SetBackwardIP{}", encryptionIP(state.backward_ip_addr)).as_bytes(), &recv_addr).await; // SetBackwardIP backIP
                        match ret { Ok(_) => println!(" (Ok)"), Err(_) => println!(" (Fail)") };
                        let ret = socket.send_to(format!("\\SetForwardIP{}", encryptionIP(recv_addr)).as_bytes(), &state.backward_ip_addr).await; // SetForwardIP recv
                        match ret { Ok(_) => println!(" (Ok)"), Err(_) => println!(" (Fail)") };
                        
                        state.backward_ip_addr = SocketAddr::new(recv_addr.ip().into(), port); //
                    }
                }
                if let Some(idx) = msg_content.find("\\SetBackwardIP"){
                    if idx == 0 {
                        let (oct1, oct2, oct3, oct4, port) = decryption(&msg_content[14..].to_string());
                        state.backward_ip_addr = SocketAddr::new(Ipv4Addr::new(oct1,oct2,oct3,oct4).into(), port);
                    }
                }
                if let Some(idx) = msg_content.find("\\SetForwardIP"){
                    if idx == 0 {
                        let (oct1, oct2, oct3, oct4, port) = decryption(&msg_content[13..].to_string());
                        state.forward_ip_addr = SocketAddr::new(Ipv4Addr::new(oct1,oct2,oct3,oct4).into(), port);
                    }
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
            
            print!("Client sent: {}", send_msg);
            if state.forward_ip_addr.ip() != Ipv4Addr::new(0, 0, 0, 0) {
                let ret = socket.send_to(send_msg.as_bytes(), &state.forward_ip_addr).await;
                match ret { Ok(_) => println!(" (Ok)"), Err(_) => println!(" (Fail)") };
            }
            if state.backward_ip_addr.ip() != Ipv4Addr::new(0, 0, 0, 0) {
                let ret = socket.send_to(send_msg.as_bytes(), &state.backward_ip_addr).await;
                match ret { Ok(_) => println!(" (Ok)"), Err(_) => println!(" (Fail)") };
            }
            RecvMessage{
                who: "".to_string(),
                contents: "".to_string(),
            }.send_signal_to_dart(None);
        }
    });
    let mut _password_generate: tokio::sync::mpsc::Receiver<rinf::DartSignal<GetMyPassword>> = GetMyPassword::get_dart_signal_receiver();
    tokio::spawn(async move {
        while let Some(dart_signal) = _password_generate.recv().await {

            if let SocketAddr::V4(v4_addr) = self_recv_addr {
                let ip = v4_addr.ip();
                let port = v4_addr.port();
                let octets = ip.octets();
        
                let encrypted_ip = encryptionIP(self_recv_addr);
                
                ThisisMyPassword{
                    password: encrypted_ip,
                }.send_signal_to_dart(None);
            }
        }
    });

    let mut _enter_new_ip: tokio::sync::mpsc::Receiver<rinf::DartSignal<KnockIp>> = KnockIp::get_dart_signal_receiver();
    tokio::spawn(async move {
        let socket = UdpSocket::bind(self_send_addr).await.unwrap();
        while let Some(dart_signal) = _enter_new_ip.recv().await {
            let msg = dart_signal.message;

            let handshakemsg = "\\NiceToMeetYou";
            let passowrd = msg.password;
            let (oct1, oct2, oct3, oct4, port) = decryption(&passowrd);

            let newIP = SocketAddr::new(Ipv4Addr::new(oct1,oct2,oct3,oct4).into(), port);
            let ret = socket.send_to(handshakemsg.as_bytes(), &newIP).await;

            print!("Client sent: {}", handshakemsg);
            match ret { Ok(_) => println!(" (Ok)"), Err(_) => println!(" (Fail)") };

        }
    });
}

fn encryptionIP(ipddr: SocketAddr) -> String{

    if let SocketAddr::V4(v4_addr) = ipddr {
        let octets = v4_addr.ip().octets();
        let port = v4_addr.port();

        let mut transformed = String::new();
        let rand_num1: u8 = rand::thread_rng().gen_range(0..=25);
        let rand_num2: u8 = rand::thread_rng().gen_range(0..=25);
        let rand_num3: u8 = rand::thread_rng().gen_range(0..=25);
        transformed.push((rand_num1 as u8 + 65) as char);
        transformed.push((rand_num2 as u8 + 65) as char);
        transformed.push((((octets[0] / 26 + rand_num1) % 26) as u8 + 65) as char);
        transformed.push((((octets[0] % 26 + rand_num2) % 26) as u8 + 65) as char);
        transformed.push((((octets[1] / 26 + rand_num1) % 26) as u8 + 65) as char);
        transformed.push((((octets[1] % 26 + rand_num3) % 26) as u8 + 65) as char);
        transformed.push((((octets[2] / 26 + rand_num1) % 26) as u8 + 65) as char);
        transformed.push((((octets[2] % 26 + rand_num2) % 26) as u8 + 65) as char);
        transformed.push((((octets[3] / 26 + rand_num3) % 26) as u8 + 65) as char);
        transformed.push((((octets[3] % 26 + rand_num2) % 26) as u8 + 65) as char);
        
        transformed.push((rand_num3 as u8 + 65) as char);
        let port_str = format!("{:05}", port);
        transformed.push(((port_str.chars().nth(0).unwrap().to_digit(10).unwrap() as u8 + rand_num1) % 26 + 65) as char);
        transformed.push(((port_str.chars().nth(1).unwrap().to_digit(10).unwrap() as u8 + rand_num3) % 26 + 65) as char);
        transformed.push(((port_str.chars().nth(2).unwrap().to_digit(10).unwrap() as u8 + rand_num1) % 26 + 65) as char);
        transformed.push(((port_str.chars().nth(3).unwrap().to_digit(10).unwrap() as u8 + rand_num2) % 26 + 65) as char);
        transformed.push(((port_str.chars().nth(4).unwrap().to_digit(10).unwrap() as u8 + rand_num3) % 26 + 65) as char);
        
        println!("I'm {}.{}.{}.{}:{} - {}", octets[0], octets[1], octets[2], octets[3], port, transformed);
        
        return transformed;
    }
    return "".to_string();
}

fn decryption(entrancecode: &String) -> (u8, u8, u8, u8, u16){

    let key_num1 = entrancecode.chars().nth(0).unwrap() as u16;
    let key_num2 = entrancecode.chars().nth(1).unwrap() as u16;
    let key_num3 = entrancecode.chars().nth(10).unwrap() as u16;

    let oct11 = ((entrancecode.chars().nth(2).unwrap() as u16 + 26) - key_num1) % 26;
    let oct12 = ((entrancecode.chars().nth(3).unwrap() as u16 + 26) - key_num2) % 26;
    let oct21 = ((entrancecode.chars().nth(4).unwrap() as u16 + 26) - key_num1) % 26;
    let oct22 = ((entrancecode.chars().nth(5).unwrap() as u16 + 26) - key_num3) % 26;
    let oct31 = ((entrancecode.chars().nth(6).unwrap() as u16 + 26) - key_num1) % 26;
    let oct32 = ((entrancecode.chars().nth(7).unwrap() as u16 + 26) - key_num2) % 26;
    let oct41 = ((entrancecode.chars().nth(8).unwrap() as u16 + 26) - key_num3) % 26;
    let oct42 = ((entrancecode.chars().nth(9).unwrap() as u16 + 26) - key_num2) % 26;

    let port1 = ((entrancecode.chars().nth(11).unwrap() as u16 + 26 - key_num1)) % 26;
    let port2 = ((entrancecode.chars().nth(12).unwrap() as u16 + 26 - key_num3)) % 26;
    let port3 = ((entrancecode.chars().nth(13).unwrap() as u16 + 26 - key_num1)) % 26;
    let port4 = ((entrancecode.chars().nth(14).unwrap() as u16 + 26 - key_num2)) % 26;
    let port5 = ((entrancecode.chars().nth(15).unwrap() as u16 + 26 - key_num3)) % 26;
    
    let oct1 = oct11 * 26 + oct12;
    let oct2 = oct21 * 26 + oct22;
    let oct3 = oct31 * 26 + oct32;
    let oct4 = oct41 * 26 + oct42;
    let port = port1 * 10000 + port2 * 1000 + port3 * 100 + port4 * 10 + port5;
    
    println!("{} Solve : {}.{}.{}.{}:{}", entrancecode, oct1, oct2, oct3, oct4, port);

    return (oct1 as u8, oct2 as u8, oct3 as u8, oct4 as u8, port as u16);
}
