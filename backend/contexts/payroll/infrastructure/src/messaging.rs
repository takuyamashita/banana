//! 出来事を outbox に記録し、relay が SQS へ送る。キューに流す形(封筒とペイロード)もここで決める

pub mod envelope;
pub mod outbox;
pub mod payloads;
pub mod relay;
