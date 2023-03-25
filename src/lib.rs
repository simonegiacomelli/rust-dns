use std::net::UdpSocket;
use std::thread::Thread;
use std::{thread, vec};
use std::fmt::DebugStruct;
use std::ops::Add;

mod find_udp_port;

// https://github.com/akapila011/DNS-Server
fn start_dns_server_thread(port: u16) {
    thread::spawn(move || { start_dns_server(port) });
}

fn start_dns_server(port: u16) {
    let bind = UdpSocket::bind(("0.0.0.0", port)).unwrap();
    let mut buf: Vec<u8> = vec![0; 4096];
    loop {
        let (size, origin) = bind.recv_from(&mut buf).unwrap();
        let x = &buf[0..size];
        println!("size={} origin={} buffer=`{:02X?}`", size, origin, x)
    }
}

fn decode_domain(buf: &[u8]) -> Result<Vec<String>, String> {
    let mut index = 0;
    let mut result: Vec<String> = Vec::new();
    loop {
        let mut length = buf[index] as usize;
        if length == 0 { break; };
        index += 1;
        let slice = &buf[index..(index + length)];
        let part = std::str::from_utf8(&slice).unwrap().to_string();
        result.push(part);
        index += length;
    }
    return Ok(result);
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use crate::find_udp_port::find_port;
    use super::*;


    #[test]
    fn test_local_resolver() {
        let port = find_port().unwrap();
        start_dns_server_thread(port);

        let output = Command::new("dig")
            .args(format!("@127.0.0.1 -p {} example.com +short", port).split(" "))
            .output()
            .expect("failed to execute process");

        println!("status: {}", output.status);
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));

        let expected_ip = "1.1.1.1";
        let actual_ip = String::from_utf8_lossy(&output.stdout).to_string();

        assert!(output.status.success());
        assert_eq!(actual_ip.trim(), expected_ip)
    }

    #[test]
    fn dig_with_examplecom() {
        let mut cmd = Command::new("dig");
        let args1 = "@127.0.0.1 -p 10000 example.com +short";
        let args1 = "example.com +short";

        let output = Command::new("dig")
            .args(args1.split(" "))
            .output()
            .expect("failed to execute process");

        println!("status: {}", output.status);
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));

        let expected_ip = "93.184.216.34";
        let actual_ip = String::from_utf8_lossy(&output.stdout).to_string();

        assert!(output.status.success());
        assert_eq!(actual_ip.trim(), expected_ip)
    }

    #[test]
    fn test_decode_domain() {
        let buf: Vec<u8> = vec![0x2, 'x' as u8, 'y' as u8, 0x1, 'z' as u8, 0x0];
        let actual = decode_domain(&buf).unwrap();
        let expected = vec!["xy".to_string(), "z".to_string()];
        assert_eq!(expected, actual)
    }
}
