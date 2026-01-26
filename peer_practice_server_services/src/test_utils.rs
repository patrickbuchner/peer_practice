use tokio::sync::mpsc;
use tokio::time::{Duration, timeout};

pub const TEST_TIMEOUT: Duration = Duration::from_millis(300);
pub const SHORT_TIMEOUT: Duration = Duration::from_millis(150);

pub async fn recv_timeout<T>(rx: &mut mpsc::Receiver<T>) -> T {
    timeout(TEST_TIMEOUT, rx.recv())
        .await
        .expect("timed out")
        .expect("channel closed")
}

pub async fn recv_timeout_unbounded<T>(rx: &mut mpsc::UnboundedReceiver<T>) -> Option<T> {
    timeout(TEST_TIMEOUT, rx.recv())
        .await
        .expect("timed out")
}

pub async fn expect_no_message<T>(rx: &mut mpsc::Receiver<T>) {
    let got = timeout(SHORT_TIMEOUT, rx.recv()).await;
    assert!(got.is_err(), "expected no message");
}

pub async fn expect_no_message_unbounded<T>(rx: &mut mpsc::UnboundedReceiver<T>) {
    let got = timeout(SHORT_TIMEOUT, rx.recv()).await;
    assert!(got.is_err(), "expected no message");
}
