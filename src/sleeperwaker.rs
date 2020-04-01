use std::sync::{Arc, Mutex, Condvar};

// This code is taken straight from https://doc.rust-lang.org/stable/std/sync/struct.Condvar.html
// but wrapped up an API that only allow the sleep/wake to happen once, and makes it more obvious
// what the calling code is trying to accomplish.

type Pair = Arc<(Mutex<bool>, Condvar)>;

#[allow(clippy::mutex_atomic)]
pub fn sleeper_waker() -> (Sleeper, Waker) {
    let pair = Pair::new((Mutex::new(false), Condvar::new()));
    (Sleeper(pair.clone()), Waker(pair))
}


pub struct Sleeper(Pair);
impl Sleeper {
    pub fn sleep(self) {
        let Self(pair) = self;
        let (lock, cvar) = &*pair;
        let mut woke = lock.lock().unwrap();
        while !*woke {
            woke = cvar.wait(woke).unwrap();
        }
    }
}

pub struct Waker(Pair);
impl Waker {
    pub fn wake(self) {
        let Self(pair) = self;
        let (lock, cvar) = &*pair;
        let mut woke = lock.lock().unwrap();
        *woke = true;
        cvar.notify_one();
    }
}