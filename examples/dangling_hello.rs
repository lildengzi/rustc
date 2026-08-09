use rustc::hello_world;

#[inline(never)]
fn clobber_stack() -> usize {
    let trash = [0xEEu8; 256];
    trash.iter().map(|&b| b as usize).sum()
}

fn main() {
    let d: &'static str = hello_world();
    let noise = clobber_stack();
    println!("{d} (noise={noise})");
}
