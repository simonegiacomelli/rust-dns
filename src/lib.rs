pub fn add(left: usize, right: usize) -> usize {
    left + right
}

#[cfg(test)]
mod tests {
    use std::process::Command;
    use super::*;

    #[test]
    fn it_works() {
        let mut cmd = Command::new("dig");
        let args1 = "@127.0.0.1 -p 10000 example.com +short";
        // let args1 = "example.com";
        args1.split(" ").for_each(|s| { cmd.arg(s); });
        let mut execution = cmd.spawn().unwrap();
        let result = execution.wait().unwrap();
        assert_eq!(0, result.code().unwrap())
    }

    #[test]
    fn it_works_not() {
        let mut cmd = Command::new("dig");
        let args1 = "@127.0.0.1 -p 10000 example.com +short";
        // let args1 = "example.com";
        args1.split(" ").for_each(|s| { cmd.arg(s); });

        let output = Command::new("dig")
            .args(args1.split(" "))
            .output()
            .expect("failed to execute process");

        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

        assert!(output.status.success());
    }
}
