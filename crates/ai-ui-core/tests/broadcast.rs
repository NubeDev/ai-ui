//! Diagnose: does our broadcast channel survive a subscriber that we hold
//! across an await point? This is what the events handler does.

use std::time::Duration;

use ai_ui_core::{AiUiState, ChatStream, Provider, ProviderContext};
use ai_ui_types::{ChatMessage, PushEvent};
use futures::StreamExt;
use tokio_stream::wrappers::BroadcastStream;

struct NoopProvider;
impl Provider for NoopProvider {
    fn stream_chat(&self, _: ProviderContext, _: Vec<ChatMessage>) -> ChatStream {
        Box::pin(futures::stream::empty())
    }
}

fn state() -> AiUiState {
    AiUiState::builder().provider(NoopProvider).build().unwrap()
}

#[tokio::test]
async fn raw_subscribe_then_broadcast_works() {
    let s = state();
    let mut rx = s.subscribe();
    let n = s.broadcast(PushEvent::Ping);
    assert_eq!(n, 1, "raw broadcast should reach the receiver");
    let msg = rx.recv().await.unwrap();
    assert!(matches!(msg, PushEvent::Ping));
}

#[tokio::test]
async fn full_handler_chain_keeps_subscriber_alive() {
    use futures::stream;
    let s = state();
    let rx = s.subscribe();
    let inner = BroadcastStream::new(rx)
        .filter_map(|res| async move { res.ok() })
        .map(|event: PushEvent| serde_json::to_string(&event).unwrap_or_default());
    let chained = stream::once(async { "ready".to_string() }).chain(inner);
    // Pin it the same way Sse will under the hood.
    futures::pin_mut!(chained);

    // Drain the first item (the "ready" event), to mimic axum's first poll.
    let first = chained.next().await;
    assert_eq!(first.as_deref(), Some("ready"));

    // Wait, then broadcast. Receiver MUST still be alive.
    tokio::time::sleep(Duration::from_millis(50)).await;
    let n = s.broadcast(PushEvent::Ping);
    assert_eq!(n, 1, "receiver dropped while chained stream is alive");
}

#[tokio::test]
async fn broadcast_stream_wrapper_keeps_subscriber_alive() {
    let s = state();
    let rx = s.subscribe();
    let mut stream = BroadcastStream::new(rx);

    // Give axum-style "the SSE runtime is between polls" some slack.
    tokio::time::sleep(Duration::from_millis(50)).await;

    let n = s.broadcast(PushEvent::Ping);
    assert_eq!(n, 1, "BroadcastStream should still hold the receiver");

    let item = stream.next().await.expect("one item");
    assert!(item.is_ok(), "expected Ok(Ping)");
}
