//! outbox 経由で SQS へ送る側(relay)と、受け取った側の冪等化(`processed_events`)

pub mod envelope;
pub mod outbox;
pub mod payloads;
pub mod processed_events;
pub mod relay;
