use std::future::Future;
use tokio::task::JoinHandle;

pub fn spawn_named<F>(name: &'static str, fut: F) -> JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    #[cfg(tokio_unstable)]
    {
        tokio::task::Builder::new()
            .name(name)
            .spawn(fut)
            .unwrap_or_else(|err| panic!("failed to spawn {name} task: {err}"))
    }
    #[cfg(not(tokio_unstable))]
    {
        let _ = name;
        tokio::spawn(fut)
    }
}
