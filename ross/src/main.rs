use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub mod context;
pub mod db;
pub mod error;
pub mod types;
mod utils;

fn main() {
    let mut options = rocksdb::Options::default();
    options.create_if_missing(true);
    options.create_missing_column_families(true);
    let db: Arc<rocksdb::DB> = Arc::new(rocksdb::DB::open(&options, "path").unwrap());
    db.compact_range(None::<&[u8]>, None::<&[u8]>);

    let spinlock = Arc::new(AtomicUsize::new(1));
    let writes = Arc::new(AtomicUsize::new(0));

    let d1 = db.clone();
    let s1 = spinlock.clone();
    let w1 = writes.clone();
    thread::spawn(move || {
        while s1.load(Ordering::SeqCst) != 0 {}
        loop {
            d1.put([100], [10]).unwrap();
            w1.fetch_add(1, Ordering::SeqCst);
        }
    });

    let d2 = db.clone();
    let s2 = spinlock.clone();
    let w2 = writes.clone();
    thread::spawn(move || {
        while s2.load(Ordering::SeqCst) != 0 {}
        loop {
            d2.put([120], [0x17]).unwrap();
            w2.fetch_add(1, Ordering::SeqCst);
        }
    });

    thread::spawn(move || {
        let now = std::time::Instant::now();
        spinlock.store(0, Ordering::SeqCst);
        loop {
            std::thread::sleep(Duration::from_millis(100));
            let d = db.get([100]).unwrap();
            let t = now.elapsed().as_millis();
            let n = writes.load(Ordering::SeqCst);
            let s = (n as f64) / (t as f64);
            println!("d = {:?}, n = {}, t = {}ms, s = {}", d, n, t, s);
        }
    })
    .join()
    .unwrap();
}
