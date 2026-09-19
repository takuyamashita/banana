//! 出来事を outbox に記録する。記録した行は relay(platform-messaging)が送る。
//! キューに流すペイロードの形もここで決める

pub mod outbox;
pub mod payloads;
