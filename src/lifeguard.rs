use crate::sleeperwaker::{sleeper_waker, Sleeper, Waker};


/// Returns a pair which ensures that DropsFirst completes it's drop()
/// before DropsSecond completes it's drop(). This is done by sleeping
/// until DropsFirst is dropped. It is trivial to get into a deadlock with
/// these by attempting to drop DropsSecond before DropsFirst from the same
/// thread - so be sure you know what you're doing.
pub fn lifeguard() -> (DropsFirst, DropsSecond) {
    let (sleeper, waker) = sleeper_waker();
    (DropsFirst(Some(waker)), DropsSecond(Some(sleeper)))
}

pub struct DropsFirst(Option<Waker>);
impl Drop for DropsFirst {
    fn drop(&mut self) {
        self.0.take().unwrap().wake();
    }
}

pub struct DropsSecond(Option<Sleeper>);
impl Drop for DropsSecond {
    fn drop(&mut self) {
        self.0.take().unwrap().sleep();
    }
}