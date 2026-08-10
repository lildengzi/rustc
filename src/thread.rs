/// Spawns a new thread, running the given closure to completion.
///
/// Returns a [`JoinHandle`](std::thread::JoinHandle) that can be used to
/// wait for the thread to finish. Unlike `std::thread::spawn`, the closure
/// is not required to be `'static` — lifetimes are erased internally, so
/// closures may borrow local values freely.
///
/// # Examples
///
/// ```
/// let handle = rustcpp::spawn(|| println!("hello from a thread"));
/// handle.join().unwrap();
/// ```
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
