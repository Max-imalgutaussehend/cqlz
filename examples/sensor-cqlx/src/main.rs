//! IoT telemetry with cqlx: one struct is the schema (the "sensor" model of DataStax's data-modeling-by-example),
//! inserts are typed, and every read is a type-state query that cannot compile without its partition key.
//! Needs a node: `CQLX_NODE` (default 127.0.0.1:9042).

use cqlx::scylla::DeserializeRow;
use cqlx::{CqlEntity, CqlModel, Session};
use std::sync::Arc;
use std::time::Instant;

#[derive(CqlEntity, DeserializeRow, Debug, Clone)]
#[cqlx(table_name = "readings", keyspace = "iot")]
struct Reading {
    #[cqlx(partition_key)]
    sensor_id: String,
    /// Bucket by day so partitions stay bounded (a sensor writing once a second makes 86,400 rows per day).
    #[cqlx(partition_key)]
    day: i32,
    #[cqlx(clustering_key(order = "desc"))]
    ts: i64,
    temperature: f64,
    humidity: f64,
}

#[tokio::main]
async fn main() -> cqlx::Result<()> {
    let node = std::env::var("CQLX_NODE").unwrap_or_else(|_| "127.0.0.1:9042".into());
    let session = Arc::new(Session::connect_single_node(&node).await?);
    println!("{}", <Reading as CqlModel>::CREATE_TABLE_CQL);
    session.create_table::<Reading>().await?;

    let (sensors, days, per_day) = (50, 3, 1_000);
    let t = Instant::now();
    let sem = Arc::new(tokio::sync::Semaphore::new(128));
    let mut tasks = tokio::task::JoinSet::new();
    for s in 0..sensors {
        for d in 0..days {
            for i in 0..per_day {
                let (session, sem) = (session.clone(), sem.clone());
                tasks.spawn(async move {
                    let _p = sem.acquire().await.unwrap();
                    let r = Reading { sensor_id: format!("sensor-{s:03}"), day: 20_000 + d, ts: i64::from(i) * 86_400, temperature: 18.0 + f64::from(i % 100) / 10.0, humidity: 40.0 + f64::from(i % 50) };
                    session.insert(&r).await.unwrap();
                });
            }
        }
    }
    while tasks.join_next().await.is_some() {}
    let n = sensors * days * per_day;
    println!("inserted {n} readings in {:.2}s ({:.0}/s)", t.elapsed().as_secs_f64(), f64::from(n) / t.elapsed().as_secs_f64());

    // The type-state builder: `.sensor_id(..).day(..)` are both required before `.fetch` exists.
    let latest = Reading::query().sensor_id("sensor-007".into()).day(20_001).limit(3).fetch(&session).await?;
    println!("latest 3 of sensor-007 on day 20001: {:?}", latest.iter().map(|r| (r.ts, r.temperature)).collect::<Vec<_>>());
    assert_eq!(latest.len(), 3);
    assert!(latest[0].ts > latest[1].ts, "clustering order is ts DESC");

    let window = Reading::query().sensor_id("sensor-007".into()).day(20_001).ts_ge(100 * 86_400).ts_lt(110 * 86_400).fetch(&session).await?;
    assert_eq!(window.len(), 10);
    println!("range query returned {} rows", window.len());

    // Reading::query().sensor_id("x".into()).fetch(&session)  // does not compile: `day` is still Missing
    println!("ok");
    Ok(())
}
