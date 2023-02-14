extern crate mouse_keyboard_input;

use std::net::UdpSocket;
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

fn main() {
    let mut device = VirtualDevice::new();

    let socket = match UdpSocket::bind("0.0.0.0:5005") {
        Ok(s) => s,
        Err(e) => panic!("couldn't bind socket: {}", e)
    };

    println!("Listening at port 5005:");

    let mut msg = [0; 2];
    loop {
        let (msg_len, _) = socket.recv_from(&mut msg).unwrap();
        if msg_len == 2 {
            if msg[0] == 128 {
                let y = to_num(msg[1]);
                device.scroll_vertical(y).unwrap();
            } else if msg[1] == 128 {
                let x = to_num(msg[1]);
                device.scroll_horizontal(x).unwrap();
            } else {
                let x = to_num(msg[0]);
                let y = to_num(msg[1]);
                device.move_mouse(x, -y).unwrap();
            }
        } else if msg_len == 1 {
            if msg[0] > 128 {
                msg[0] -= 128;
                device.press(to_button(msg[0])).unwrap();
            } else {
                device.release(to_button(msg[0])).unwrap();
            }
        }
    }
}