use rustc::spawn_cpp_dangerous;
use std::thread;

fn main() {
    let message = String::from("thread says hi");
    let handle = spawn_cpp_dangerous(|| {
        thread::sleep(std::time::Duration::from_millis(10));
        println!("{message}");
    });
    drop(message);
    handle.join().unwrap();
}
