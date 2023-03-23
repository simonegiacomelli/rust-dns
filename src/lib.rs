use std::net::UdpSocket;
use std::thread::Thread;
use std::{thread, vec};

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
}
