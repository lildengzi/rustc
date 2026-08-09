pub fn spawn_cpp_dangerous<F>(f: F) -> std::thread::JoinHandle<()>
where
    F: FnOnce(),
{
    let boxed: Box<dyn FnOnce() + '_> = Box::new(f);
    let static_boxed: Box<dyn FnOnce() + Send + 'static> =
        unsafe { std::mem::transmute(boxed) };
    std::thread::spawn(move || static_boxed())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spawns_and_joins() {
        let handle = spawn_cpp_dangerous(|| println!("hello from thread"));
        handle.join().unwrap();
    }
}
