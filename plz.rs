use std::env;
use std::io::{self, BufRead};
use std::process::{Command, Stdio};

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut command = Command::new("python");
    if args.len() > 1 {
        command.arg("D:/Programming/Python/plz/plz.py").args(&args[1..]);
    } else {
        command.arg("D:/Programming/Python/plz/plz.py");
    }

    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command.spawn().expect("Failed to start command");

    let stdout_child = child.stdout.take().expect("Failed to get stdout");
    let stderr_child = child.stderr.take().expect("Failed to get stderr");

    let stdout_thread = std::thread::spawn(move || {
        let reader = io::BufReader::new(stdout_child);
        for line in reader.lines() {
            match line {
                Ok(line) => println!("{}", line),
                Err(e) => eprintln!("Failed to read line from stdout: {:?}", e),
            }
        }
    });
    let stderr_thread = std::thread::spawn(move || {
        let reader = io::BufReader::new(stderr_child);
        for line in reader.lines() {
            match line {
                Ok(line) => eprintln!("{}", line),
                Err(e) => eprintln!("Failed to read line from stderr: {:?}", e),
            }
        }
    });

    child.wait().expect("Failed to wait for command");
    stdout_thread.join().expect("Failed to join stdout thread");
    stderr_thread.join().expect("Failed to join stderr thread");
}
