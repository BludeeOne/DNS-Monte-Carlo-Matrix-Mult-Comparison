//https://stackoverflow.com/questions/57256035/how-to-lock-a-rust-struct-the-way-a-struct-is-locked-in-go
use std::sync::{Arc, RwLock};
// https://doc.rust-lang.org/std/sync/struct.Arc.html
// https://doc.rust-lang.org/std/sync/struct.RwLock.html
use std::thread;


#[derive(Debug, Default)]
struct SafeInt {
    data: RwLock<IntStruct>,
}

#[derive(Debug, Default)]
struct IntStruct {
    value: i32,
}

fn main() {
    let info = Arc::new(SafeInt::default());
    let mut children = vec![];
    for _n in 0..1000 {
        // https://stackoverflow.com/questions/27359586/what-do-i-use-to-share-an-object-with-many-threads-and-one-writer-in-rust
        let shared_data = info.clone();
		children.push(thread::spawn(move || {
            let mut data = shared_data.data.write().expect("Lock is poisoned");
            data.value += 1;
        }));
    }
    for child in children {
        // Wait for the thread to finish. Returns a result.
        let _ = child.join();
    }
    let data = info.data.read().expect("Lock is poisoned");
    println!("Value is {}", data.value);
}
