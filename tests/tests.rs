use fn_to_fut::fn_to_fut;
use std::thread;
use tokio::runtime::Runtime;
use std::future::Future;

fn block_on<F: Future>(f: F) -> F::Output {
    let mut runtime = Runtime::new().unwrap();
    runtime.block_on(f)
}

#[test]
fn test() {
    let result = block_on(async move {
        let shared_data = vec![1, 2];
        //let shared_ref = &shared_data[1];
        let (fut, work) = fn_to_fut(|| shared_data[0]);
        thread::spawn(work);
        fut.await
    });
    assert_eq!(result, 3);
}