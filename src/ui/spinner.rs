use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn run<F>(msg: &str, f: F) -> anyhow::Result<()>
where
    F: FnOnce() -> anyhow::Result<()>,
{
    let stop = Arc::new(AtomicBool::new(false));
    let stop_clone = stop.clone();
    let msg_owned = msg.to_string();

    let handle = thread::spawn(move || {
        let chars = ['-', '/', '|', '\\'];
        let mut i = 0;
        while !stop_clone.load(Ordering::Relaxed) {
            eprint!("\r{} {}", msg_owned, chars[i % chars.len()]);
            i += 1;
            thread::sleep(Duration::from_millis(100));
        }
        eprint!("\r{}\r", " ".repeat(msg_owned.len() + 4));
        eprintln!("{} {}", msg_owned, crate::ui::color::green("✓ 完成"));
    });

    let result = f();
    stop.store(true, Ordering::Relaxed);
    handle.join().unwrap();
    result
}
