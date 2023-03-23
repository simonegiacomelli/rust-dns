use std::borrow::BorrowMut;
use std::io::repeat;
use std::net::{TcpListener, UdpSocket};
use std::ops::Range;
use std::sync::atomic::{AtomicU16, Ordering};

static last_port: AtomicU16 = AtomicU16::new(10000);


pub fn find_port() -> Option<u16> {
    loop {
        let port = last_port.fetch_add(1, Ordering::SeqCst);
        if port_is_available(port) { return Some(port); }
        if port > 20000 { return None; }
    }
}

pub fn find_port_in(range: Range<u16>) -> Option<u16> {
    let mut range1 = range.clone();
    range1.find(|port| port_is_available(*port))
}

fn port_is_available(port: u16) -> bool {
    match UdpSocket::bind(("0.0.0.0", port)) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[cfg(test)]
mod test {
    use super::find_port;

    #[test]
    fn test_no_reuse() {
        let p1 = find_port().unwrap();
        let p2 = find_port().unwrap();
        println!("p1={} p2={}", p1, p2);
        assert_ne!(p1, p2)
    }
}