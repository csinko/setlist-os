use lapin::{Channel, options::*, types::FieldTable};
use anyhow::Result;

pub const EXCHANGE: &str = "jobs";

pub const QUEUES: &[(&str, &str)] = &[
    ("queue.import",      "import"),
    ("queue.fingerprint", "fingerprint"),
    ("queue.match_track", "match_track"),
    ("queue.match_album", "match_album"),
    ("queue.tag_track",   "tag_track"),
    ("queue.index",       "index"),
];

pub async fn declare_all(ch: &Channel) -> Result<()> {
    ch.exchange_declare(
        EXCHANGE,
        lapin::ExchangeKind::Direct,
        ExchangeDeclareOptions { durable: true, ..Default::default() },
        FieldTable::default(),
    ).await?;
    for &(queue, rk) in QUEUES {
        ch.queue_declare(
            queue,
            QueueDeclareOptions { durable: true, ..Default::default() },
            FieldTable::default(),
        ).await?;
        ch.queue_bind(
            queue,
            EXCHANGE,
            rk,
            QueueBindOptions::default(),
            FieldTable::default(),
        ).await?;
    }
    Ok(())
}

