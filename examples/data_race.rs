use rustcpp::*;
use std::thread;

fn main() {
    let rc = Rc::new(0);
    let mut handles = Vec::new();
    for _ in 0..8 {
        let rc = rc.clone();
        handles.push(thread::spawn(move || {
            for _ in 0..100_000 {
                let _ = rc.clone();
                drop(rc.clone());
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    println!("final count: {}", Rc::strong_count(&rc));
}
