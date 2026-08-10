use rustcpp::vec;

fn main() {
    let mut v = vec![1, 2, 3];
    for &x in &v {
        println!("item: {x}");
        if x == 2 {
            v.push(99);
            println!("pushed while iterating");
        }
    }
}
