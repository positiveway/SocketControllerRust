extern crate mouse_keyboard_input;

use std::borrow::BorrowMut;
use std::io::Read;
use std::net::{UdpSocket, TcpListener, TcpStream, Shutdown};
use std::sync::{Arc, Mutex};
use std::thread;
use std::thread::JoinHandle;
use mouse_keyboard_input::VirtualDevice;
use mouse_keyboard_input::key_codes::*;


fn to_num(one_byte: u8) -> i32 {
    let mut num = one_byte as i32;
    if num > 128 {
        num -= 256;
    }
    return num;
}

const LEFT_MOUSE: u8 = 90;
const RIGHT_MOUSE: u8 = 91;
const MIDDLE_MOUSE: u8 = 92;

fn to_button(one_byte: u8) -> u16 {
    match one_byte {
        LEFT_MOUSE => BTN_LEFT,
        RIGHT_MOUSE => BTN_RIGHT,
        MIDDLE_MOUSE => BTN_MIDDLE,
        _ => one_byte as u16,
    }
}
//
// fn handle_client(mut stream: TcpStream, device: &mut VirtualDevice) {
//     let mut msg = [0 as u8; 1]; // using 1 byte buffer
//     while match stream.read(&mut msg) {
//         Ok(size) => {
//             if msg[0] == 128 {
//                 println!("Terminating connection with {}", stream.peer_addr().unwrap());
//                 return;
//             } else if msg[0] > 128 {
//                 msg[0] -= 128;
//                 device.press(to_button(msg[0])).unwrap();
//             } else {
//                 device.release(to_button(msg[0])).unwrap();
//             }
//             true
//         }
//         Err(_) => {
//             println!("An error occurred, terminating connection with {}", stream.peer_addr().unwrap());
//             stream.shutdown(Shutdown::Both).unwrap();
//             false
//         }
//     } {}
// }
//
// fn handle_tcp() {
//     let tcp_listener = TcpListener::bind("0.0.0.0:5007").unwrap();
//     // accept connections and process them, spawning a new thread for each one
//     println!("TCP at port 5007");
//
//     let mut device = VirtualDevice::new();
//
//     for stream in tcp_listener.incoming() {
//         match stream {
//             Ok(stream) => {
//                 println!("New connection: {}", stream.peer_addr().unwrap());
//                 // connection succeeded
//                 handle_client(stream, device.borrow_mut())
//             }
//             Err(e) => {
//                 println!("Error: {}", e);
//                 /* connection failed */
//             }
//         }
//     }
//
//     // close the socket server
//     drop(tcp_listener);
// }

type SharedDevice = Arc<Mutex<VirtualDevice>>;

fn parse_mouse(socket: UdpSocket, device: SharedDevice) {
    let mut msg = [0; 2];
    loop {
        socket.recv_from(&mut msg).unwrap();
        let mut locked_device = device.lock().unwrap();

        let x = to_num(msg[0]);
        let y = to_num(msg[1]);
        locked_device.move_mouse(x, -y).unwrap();
    }
}

fn parse_scroll(socket: UdpSocket, device: SharedDevice) {
    let mut msg = [0; 1];
    loop {
        socket.recv_from(&mut msg).unwrap();
        let mut locked_device = device.lock().unwrap();

        let y = to_num(msg[0]);
        locked_device.scroll_vertical(y).unwrap();
    }
}

fn parse_button(socket: UdpSocket, device: SharedDevice) {
    let mut msg = [0; 1];
    loop {
        socket.recv_from(&mut msg).unwrap();
        let mut locked_device = device.lock().unwrap();

        if msg[0] > 128 {
            msg[0] -= 128;
            locked_device.press(to_button(msg[0])).unwrap();
        } else {
            locked_device.release(to_button(msg[0])).unwrap();
        }
    }
}

fn create_udp_thread(parse_func: fn(UdpSocket, SharedDevice), port: u16, device: SharedDevice) -> JoinHandle<()> {
    thread::spawn(move || {
        // let address = format!("0.0.0.0:{}", port);
        let address = "0.0.0.0";

        let socket = match UdpSocket::bind((address, port)) {
            Ok(s) => s,
            Err(e) => panic!("couldn't bind socket: {}", e)
        };

        println!("UDP at port {}:", port);

        parse_func(socket, device);
    })
}

fn main() {
    let mut device = Arc::new(Mutex::new(VirtualDevice::new()));

    create_udp_thread(parse_button, 5009, device.clone());
    create_udp_thread(parse_scroll, 5007, device.clone());
    create_udp_thread(parse_mouse, 5005, device.clone()).join().unwrap();
}