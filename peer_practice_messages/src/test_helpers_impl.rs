use chrono::TimeZone;
use tokio::sync::mpsc;
use tokio::sync::mpsc::error::TryRecvError;
use tokio::sync::oneshot;
use tokio::sync::oneshot::error::TryRecvError as TryRecvOneshotError;
use tokio::task::yield_now;

const MAX_YIELDS: usize = 128;
const NO_MESSAGE_YIELDS: usize = 32;

#[allow(dead_code)]
pub fn fixed_timestamp() -> chrono::DateTime<chrono::Utc> {
    chrono::Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap()
}

#[allow(dead_code)]
pub async fn recv_timeout<T>(rx: &mut mpsc::Receiver<T>) -> T {
    for _ in 0..MAX_YIELDS {
        match rx.try_recv() {
            Ok(msg) => return msg,
            Err(TryRecvError::Empty) => yield_now().await,
            Err(TryRecvError::Disconnected) => panic!("channel closed"),
        }
    }
    panic!("timed out")
}

#[allow(dead_code)]
pub async fn recv_timeout_unbounded<T>(rx: &mut mpsc::UnboundedReceiver<T>) -> Option<T> {
    for _ in 0..MAX_YIELDS {
        match rx.try_recv() {
            Ok(msg) => return Some(msg),
            Err(TryRecvError::Empty) => yield_now().await,
            Err(TryRecvError::Disconnected) => return None,
        }
    }
    panic!("timed out")
}

#[allow(dead_code)]
pub async fn expect_no_message<T>(rx: &mut mpsc::Receiver<T>) {
    for _ in 0..NO_MESSAGE_YIELDS {
        match rx.try_recv() {
            Ok(_) => panic!("expected no message"),
            Err(TryRecvError::Empty) => yield_now().await,
            Err(TryRecvError::Disconnected) => panic!("channel closed"),
        }
    }
}

#[allow(dead_code)]
pub async fn expect_no_message_unbounded<T>(rx: &mut mpsc::UnboundedReceiver<T>) {
    for _ in 0..NO_MESSAGE_YIELDS {
        match rx.try_recv() {
            Ok(_) => panic!("expected no message"),
            Err(TryRecvError::Empty) => yield_now().await,
            Err(TryRecvError::Disconnected) => panic!("channel closed"),
        }
    }
}

#[allow(dead_code)]
pub async fn recv_oneshot_timeout<T>(mut rx: oneshot::Receiver<T>) -> T {
    for _ in 0..MAX_YIELDS {
        match rx.try_recv() {
            Ok(value) => return value,
            Err(TryRecvOneshotError::Empty) => yield_now().await,
            Err(TryRecvOneshotError::Closed) => panic!("oneshot closed"),
        }
    }
    panic!("timed out")
}
