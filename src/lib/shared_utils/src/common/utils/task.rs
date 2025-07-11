use std::pin::Pin;
use futures::{stream::FuturesUnordered, Future, StreamExt};

pub async fn run_task_concurrently<T>(
    mut futures: impl Iterator<Item = impl Future<Output = T>>, 
    concurrency: usize,
    mut result_callback: impl FnMut(T), 
    breaking_condition: impl Fn() -> bool)   {


        let mut in_progress_futures: FuturesUnordered<Pin<Box<dyn Future<Output = T>>>> = FuturesUnordered::new();

        for _ in 0..concurrency {
            let next_future = match futures.next() {
                None => break,
                Some(some)=> some,
            };
            if breaking_condition() {
                break;
            }
            in_progress_futures.push(Box::pin(next_future));
        }

        for next_future in futures {
            if breaking_condition() {
                break;
            }
            let result = in_progress_futures.next().await.unwrap();
            result_callback(result);
            in_progress_futures.push(Box::pin(next_future));
        }

        loop {
            match in_progress_futures.next().await {
                None => break,
                Some (result) => result_callback(result)
            }
        }

}