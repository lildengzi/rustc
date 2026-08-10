use rustcpp::*;

fn main() {
    let d: &'static str;
    {
        let s = String::from("hello, dangling world");
        d = s.as_str();
        println!("before drop: {d}");
    }
    println!("after drop:  {d}");
}
