use std::future::Future;
use boxfnonce::SendBoxFnOnce;
use std::mem::transmute;
use futures::channel::oneshot;

mod sleeperwaker;
mod lifeguard;
mod error;
pub use error::Error;
use lifeguard::{lifeguard, DropsSecond};

// TODO: If the SenderFn is dropped (without being called)
// send a result of uncalled over the channel

// TODO: If the ReceiverFuture is dropped, synchronously wait for SenderFn

// TODO: If SenderFn panics, send the result

// TODO: Run miri and asan



struct SenderFn<T> {
    inner: Option<SendBoxFnOnce<'static, (), T>>,
    sender: Option<oneshot::Sender<Result<T, Error>>>,
}


impl<T> SenderFn<T> {
    fn new(sender: oneshot::Sender<Result<T, Error>>, f: SendBoxFnOnce<'static, (), T>) -> Self {
        Self {
            inner: Some(f),
            sender: Some(sender),
        }
    }
    fn call(mut self) {
        // Taking the inner value out is necessary to indicate to drop()
        // that this function has been called.
        let inner = self.inner.take().unwrap();
        let t = inner.call();
        
        self.send_once(Ok(t));
    }

    fn send_once(&mut self, result: Result<T, Error>) {
        if let Some(sender) = self.sender.take() {
            // Ignore the error because from this end we don't care if the result is
            // no longer relevant to the caller.
            let _ignore_err = sender.send(result);
        }
        
    }
}

impl<T> Drop for SenderFn<T> {
    fn drop(&mut self) {
        // Ensure that something is always sent across the channel,
        // even if it's just an error
        let err = if self.inner.is_some() {
            Error::Uncalled
        } else {
            Error::Panic
        };
        self.send_once(Err(err));
    }
}

// Safety: This is needed to ensure that even if the future is not awaited on and dropped,
// that we still end up waiting for the Fn to be called before proceeding.
async fn receive<T>(receiver: oneshot::Receiver<Result<T, Error>>, guard: DropsSecond) -> T {
    let result = receiver.await.unwrap();
    drop(guard);
    // This unwrap is ok because we want to propagate panics, and panic if the
    // future is not called
    result.unwrap()
}

// TODO: Don't use dyn in the result types
// TODO: Completely understand Unpin
pub fn fn_to_fut<'a, T: 'static>(f: impl 'a + (FnOnce() -> T) + Send + Unpin) -> (impl 'a + Future<Output=T>, impl 'static + FnOnce()) {
    let (drops_first, drops_second) = lifeguard();

    // Wrap f such that drops_second cannot be dropped
    // unless f() is dropped. In this way we unsure that
    // drops_second outlives f
    let f = move || {
        let result = f();
        drop(drops_first);
        result
    };
    
    // The need to box the future is uncertain. The reason it's done here
    // is because we need to transmute away the lifetime, but it is unclear
    // which type to transmute to without the box.
    let inner: SendBoxFnOnce<'_, (), T> = SendBoxFnOnce::new(f);
    let inner: SendBoxFnOnce<'static, (), T> = unsafe { transmute(inner) };
    let (sender, receiver) = oneshot::channel();
    let sender_fn = SenderFn::new(sender, inner);


    (receive(receiver, drops_second), move || sender_fn.call())
}
