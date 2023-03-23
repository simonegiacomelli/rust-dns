mod find_udp_port;

pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use super::*;


    #[test]
    fn it_works_not() {
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
