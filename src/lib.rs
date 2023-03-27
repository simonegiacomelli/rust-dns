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

fn build_response(query_buf: &[u8]) -> Vec<u8> {
    // let mut result = example_com_response();
    // result[0] = buf[0];
    // result[1] = buf[1];
    // return result;
    let mut result = Vec::new();
    let query = decode_query(query_buf);
    let question_index_end = query.question_index_end();
    result.extend_from_slice(&query_buf[0..question_index_end]);
    result[2] = 0x84;
    result[3] = 0x00;
    result[7] = 0x01;
    result[11] = 0x00;
    result.extend_from_slice(&vec![0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01]);
    result.extend_from_slice(&vec![0x00, 0x00, 0x02, 0x58]); // valid for 600 secs
    result.extend_from_slice(&vec![0x00, 0x04, 0x01, 0x01, 0x01, 0x01]);
    result
}

fn example_com_response() -> Vec<u8> { vec![0x18, 0x23, 0x84, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x07, 0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, 0x03, 0x63, 0x6f, 0x6d, 0x00, 0x00, 0x01, 0x00, 0x01, 0xc0, 0x0c, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x01, 0x90, 0x00, 0x04, 0x01, 0x01, 0x01, 0x01] }

struct Question {
    qname: Vec<String>,
    qtype: u16,
    qclass: u16,
}

impl Question {
    fn size(&self) -> usize {
        let qname = &self.qname;
        let qname_len: usize = qname.into_iter().map(|part| { part.len() }).sum();
        qname_len + qname.len() + 1 + 2 + 2
    }
}

trait QuestionSize {
    fn size(&self) -> usize;
}

impl QuestionSize for Vec<Question> {
    fn size(&self) -> usize {
        self.into_iter().map(|i| { i.size() }).sum()
    }
}

fn decode_questions(buf: &[u8]) -> Result<Vec<Question>, String> {
    let mut index = 0;
    let mut result: Vec<String> = Vec::new();
    loop {
        let mut length = buf[index] as usize;
        index += 1;
        if length == 0 { break; };
        let slice = &buf[index..(index + length)];
        let qname = std::str::from_utf8(&slice).unwrap().to_string();
        result.push(qname);
        index += length;
    }
    let qtype = (buf[index] as u16) << 8 | buf[index + 1] as u16;
    let qclass = (buf[index + 2] as u16) << 8 | buf[index + 3] as u16;
    return Ok(vec![Question { qname: result, qtype, qclass }]);
}

struct Query {
    questions: Vec<Question>,
}

impl Query {
    fn question_index_start() -> usize { 12 }
    fn question_index_end(&self) -> usize { Query::question_index_start() + &self.questions.size() }
}

fn decode_query(buf: &[u8]) -> Query {
    Query { questions: decode_questions(&buf[Query::question_index_start()..]).unwrap() }
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use crate::find_udp_port::find_port;
    use super::*;

    fn example_com_query() -> Vec<u8> { vec![0xE2, 0x61, 0x01, 0x20, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x07, 0x65, 0x78, 0x61, 0x6D, 0x70, 0x6C, 0x65, 0x03, 0x63, 0x6F, 0x6D, 0x00, 0x00, 0x01, 0x00, 0x01, 0x00, 0x00, 0x29, 0x04, 0xD0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x0C, 0x00, 0x0A, 0x00, 0x08, 0x1D, 0xB6, 0x2D, 0x09, 0x30, 0xD8, 0x1A, 0xFB] }

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
    fn test_decode_questions() {
        let buf: Vec<u8> = vec![0x2, 'x' as u8, 'y' as u8, 0x1, 'z' as u8, 0x0, 0x01, 0x56, 0x02, 0xFF];
        let questions = decode_questions(&buf).unwrap();
        assert_eq!(1, questions.len());
        let question = &questions[0];
        let expected = &vec!["xy".to_string(), "z".to_string()];
        assert_eq!(expected, &question.qname);
        assert_eq!(342, question.qtype);
        assert_eq!(767, question.qclass);
        assert_eq!(3 + 2 + 1 + 2 + 2, questions.size())
    }

    #[test]
    fn test_decode_query() {
        let query = decode_query(&example_com_query());
        let questions = query.questions;
        assert_eq!(1, questions.len());
        let actual = &questions[0].qname;
        let expected = &vec!["example".to_string(), "com".to_string()];
        assert_eq!(expected, actual)
    }
}
