use rustcpp::*;
use std::thread;

fn main() {
    let message = String::from("thread says hi");
    let handle = spawn(|| {
        thread::sleep(std::time::Duration::from_millis(10));
        println!("{message}");
    });
    drop(message);
    handle.join().unwrap();
}
