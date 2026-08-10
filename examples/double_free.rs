use rustc::*;

fn main() {
    let a = Box::new(String::from("boom"));
    let b = a.clone();
    drop(a);
    println!("first drop done");
    drop(b);
    println!("unreachable");
}
