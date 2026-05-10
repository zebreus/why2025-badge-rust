use std::{
    cell::RefCell,
    sync::{
        Arc, Condvar, Mutex,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

thread_local! {
    static THREAD_LOCAL_VALUE: RefCell<Option<&'static str>> = const { RefCell::new(None) };
}

fn main() {
    println!("BadgeVMS std thread smoke starting");

    let tls_dropped = Arc::new(AtomicBool::new(false));
    let tls_dropped_by_thread = Arc::clone(&tls_dropped);
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair_for_thread = Arc::clone(&pair);

    let handle = thread::Builder::new()
        .name("badge-std-worker".to_owned())
        .stack_size(16 * 1024)
        .spawn(move || {
            struct MarkDrop(Arc<AtomicBool>);
            impl Drop for MarkDrop {
                fn drop(&mut self) {
                    self.0.store(true, Ordering::SeqCst);
                }
            }

            let _marker = MarkDrop(tls_dropped_by_thread);
            THREAD_LOCAL_VALUE.with(|value| *value.borrow_mut() = Some("worker"));

            let (lock, condvar) = &*pair_for_thread;
            let mut ready = lock.lock().expect("worker lock should not be poisoned");
            *ready = true;
            condvar.notify_one();

            thread::park_timeout(Duration::from_millis(1));
            42
        })
        .expect("thread spawn should succeed");

    let (lock, condvar) = &*pair;
    let mut ready = lock.lock().expect("main lock should not be poisoned");
    while !*ready {
        ready = condvar
            .wait(ready)
            .expect("condvar wait should not be poisoned");
    }

    handle.thread().unpark();
    let result = handle.join().expect("BadgeVMS v1 aborts on thread panic");
    println!("worker result: {result}");
    println!(
        "tls destructor observed: {}",
        tls_dropped.load(Ordering::SeqCst)
    );
    println!(
        "available parallelism: {}",
        thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(0)
    );
}
