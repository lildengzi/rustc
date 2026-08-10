pub fn spawn<F>(f: F) -> std::thread::JoinHandle<()>
where
    F: FnOnce(),
{
    let boxed: std::boxed::Box<dyn FnOnce() + '_> = std::boxed::Box::new(f);
    let static_boxed: std::boxed::Box<dyn FnOnce() + Send + 'static> =
        unsafe { std::mem::transmute(boxed) };
    std::thread::spawn(move || static_boxed())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawns_and_joins() {
        let handle = spawn(|| println!("hello from thread"));
        handle.join().unwrap();
    }
}
