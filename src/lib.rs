use std::net::UdpSocket;
use std::thread::Thread;
use std::{thread, vec};
use std::fmt::DebugStruct;
use std::ops::Add;

mod find_udp_port;

// https://github.com/akapila011/DNS-Server
// https://mislove.org/teaching/cs4700/spring11/handouts/project1-primer.pdf

fn start_dns_server_thread(port: u16) {
    thread::spawn(move || { start_dns_server(port) });
}

fn start_dns_server(port: u16) {
    let socket = UdpSocket::bind(("0.0.0.0", port)).unwrap();
    let mut buf: Vec<u8> = vec![0; 4096];
    loop {
        let (size, origin) = socket.recv_from(&mut buf).unwrap();
        let x = &buf[0..size];
        println!(" origin={} size={} buffer={:02X?}", origin, x.len(), x);
        let response = build_response(x);
        println!("response size={} buffer={:02X?}", response.len(), response);

        socket.send_to(&response, origin).unwrap();
    }
}

fn build_response(buf: &[u8]) -> Vec<u8> {
    let mut result = Vec::new();
    result.extend_from_slice(&buf);
    result.extend_from_slice(&vec![0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01]);
    result.extend_from_slice(&vec![0x00, 0x00, 0x02, 0x58]); // valid for 600 secs
    result.extend_from_slice(&vec![0x00, 0x04, 0x01, 0x01, 0x01, 0x01]);
    result
}

struct Question {
    qname: Vec<String>,
}

fn decode_questions(buf: &[u8]) -> Result<Vec<Question>, String> {
    let mut index = 0;
    let mut result: Vec<String> = Vec::new();
    loop {
        let mut length = buf[index] as usize;
        if length == 0 { break; };
        index += 1;
        let slice = &buf[index..(index + length)];
        let qname = std::str::from_utf8(&slice).unwrap().to_string();
        result.push(qname);
        index += length;
    }
    return Ok(vec![Question { qname: result }]);
}

struct Query {
    questions: Vec<Question>,
}

fn decode_query(buf: &[u8]) -> Query {
    Query { questions: decode_questions(&buf[12..]).unwrap() }
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use crate::find_udp_port::find_port;
    use super::*;

    fn packet_example_com() -> Vec<u8> { vec![0xE2, 0x61, 0x01, 0x20, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x07, 0x65, 0x78, 0x61, 0x6D, 0x70, 0x6C, 0x65, 0x03, 0x63, 0x6F, 0x6D, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x29, 0x04, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x0A, 0x00, 0x08, 0x1D, 0xB6, 0x2D, 0x09, 0x30, 0xD8, 0x1A, 0xFB] }

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
    fn dig_with_example_com() {
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
        let questions = decode_questions(&buf).unwrap();
        assert_eq!(1, questions.len());
        let actual = &questions[0].qname;
        let expected = &vec!["xy".to_string(), "z".to_string()];
        assert_eq!(expected, actual)
    }

    #[test]
    fn test_decode_query() {
        let query = decode_query(&packet_example_com());
        let questions = query.questions;
        assert_eq!(1, questions.len());
        let actual = &questions[0].qname;
        let expected = &vec!["example".to_string(), "com".to_string()];
        assert_eq!(expected, actual)
    }
}
