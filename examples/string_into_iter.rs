use rustcpp::*;

fn main() {
    let s = String::from("hello world");
    for c in s {
        println!("{c}");
    }
}
