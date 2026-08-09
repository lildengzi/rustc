use rustc::CppBox;

fn main() {
    let a = CppBox::new(String::from("boom"));
    let b = a.clone();
    drop(a);
    println!("first drop done");
    drop(b);
    println!("unreachable");
}
