//! YCSB core workloads A, B and C (Zipfian 0.99) against ScyllaDB in Docker:
//! cqlx vs the raw ScyllaDB driver vs charybdis, closed loop, 64 concurrent tasks.
//!
//! Deviations from canonical YCSB, stated up front: records have 10 fields of 100 bytes as in YCSB, but an
//! *update* rewrites the whole row (cqlx has no partial update yet; the same is done for all contenders), keys
//! are not hash-scrambled, and this is one node on a laptop, not a cluster.
//!
//!     cargo run --release --bin ycsb -- [records] [seconds]

use charybdis::macros::charybdis_model;
use charybdis::operations::{Find, Insert};
use charybdis::types::Text;
use cqlx::scylla::DeserializeRow;
use cqlx::{CqlEntity, CqlModel, Session};
use hdrhistogram::Histogram;
use rand::prelude::*;
use rand_distr::Zipf;
use std::future::Future;
use std::sync::Arc;
use std::time::{Duration, Instant};
use testcontainers::core::{IntoContainerPort, WaitFor};
use testcontainers::runners::AsyncRunner;
use testcontainers::{ContainerAsync, GenericImage, ImageExt};

#[derive(CqlEntity, DeserializeRow, Clone, Debug)]
#[cqlx(table_name = "usertable", keyspace = "ycsb")]
struct UserX {
    #[cqlx(primary_key)]
    ycsb_key: String,
    field0: String,
    field1: String,
    field2: String,
    field3: String,
    field4: String,
    field5: String,
    field6: String,
    field7: String,
    field8: String,
    field9: String,
}

#[derive(DeserializeRow)]
#[allow(dead_code)]
struct RawRow {
    ycsb_key: String,
    field0: String,
    field1: String,
    field2: String,
    field3: String,
    field4: String,
    field5: String,
    field6: String,
    field7: String,
    field8: String,
    field9: String,
}

#[charybdis_model(table_name = usertable, partition_keys = [ycsb_key], clustering_keys = [], global_secondary_indexes = [], local_secondary_indexes = [], static_columns = [])]
#[derive(Default, Clone)]
struct UserC {
    ycsb_key: Text,
    field0: Text,
    field1: Text,
    field2: Text,
    field3: Text,
    field4: Text,
    field5: Text,
    field6: Text,
    field7: Text,
    field8: Text,
    field9: Text,
}

fn payload(seed: u64) -> String {
    let mut r = StdRng::seed_from_u64(seed);
    (0..100).map(|_| char::from(b'a' + r.random_range(0..26))).collect()
}

fn userx(key: String, seed: u64) -> UserX {
    let p = payload(seed);
    UserX { ycsb_key: key, field0: p.clone(), field1: p.clone(), field2: p.clone(), field3: p.clone(), field4: p.clone(), field5: p.clone(), field6: p.clone(), field7: p.clone(), field8: p.clone(), field9: p }
}

fn userc(key: String, seed: u64) -> UserC {
    let p = payload(seed);
    UserC { ycsb_key: key, field0: p.clone(), field1: p.clone(), field2: p.clone(), field3: p.clone(), field4: p.clone(), field5: p.clone(), field6: p.clone(), field7: p.clone(), field8: p.clone(), field9: p }
}

struct Stats {
    ops: u64,
    secs: f64,
    p50: u64,
    p99: u64,
}

/// Closed-loop driver: `tasks` workers draw Zipfian keys, do a read with probability `read_frac`, else a write.
async fn drive<R, RF, W, WF>(tasks: usize, secs: u64, records: u64, read_frac: f64, read: Arc<R>, write: Arc<W>) -> Stats
where
    R: Fn(String) -> RF + Send + Sync + 'static,
    RF: Future<Output = ()> + Send,
    W: Fn(String, u64) -> WF + Send + Sync + 'static,
    WF: Future<Output = ()> + Send,
{
    let deadline = Instant::now() + Duration::from_secs(secs);
    let start = Instant::now();
    let mut set = tokio::task::JoinSet::new();
    for t in 0..tasks {
        let (read, write) = (read.clone(), write.clone());
        set.spawn(async move {
            let mut rng = StdRng::seed_from_u64(1000 + t as u64);
            let zipf = Zipf::new(records as f64, 0.99).unwrap();
            let mut h = Histogram::<u64>::new_with_bounds(1, 60_000_000, 3).unwrap();
            let mut n = 0u64;
            while Instant::now() < deadline {
                let key = format!("user{}", zipf.sample(&mut rng) as u64);
                let t0 = Instant::now();
                if rng.random_bool(read_frac) {
                    read(key).await;
                } else {
                    write(key, rng.random()).await;
                }
                h.record(t0.elapsed().as_micros().max(1) as u64).ok();
                n += 1;
            }
            (n, h)
        });
    }
    let mut all = Histogram::<u64>::new_with_bounds(1, 60_000_000, 3).unwrap();
    let mut ops = 0;
    while let Some(r) = set.join_next().await {
        let (n, h) = r.unwrap();
        ops += n;
        all.add(h).unwrap();
    }
    Stats { ops, secs: start.elapsed().as_secs_f64(), p50: all.value_at_quantile(0.5), p99: all.value_at_quantile(0.99) }
}

async fn start() -> (ContainerAsync<GenericImage>, String) {
    let c = GenericImage::new("scylladb/scylla", "latest")
        .with_exposed_port(9042.tcp())
        .with_wait_for(WaitFor::message_on_stderr("serving"))
        .with_cmd(["--smp", "2", "--memory", "1500M", "--overprovisioned", "1", "--developer-mode", "1"])
        .with_startup_timeout(Duration::from_secs(240))
        .start()
        .await
        .unwrap();
    let node = format!("{}:{}", c.get_host().await.unwrap(), c.get_host_port_ipv4(9042).await.unwrap());
    (c, node)
}

#[tokio::main]
async fn main() {
    let records: u64 = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(50_000);
    let secs: u64 = std::env::args().nth(2).and_then(|s| s.parse().ok()).unwrap_or(10);
    let tasks = 64;
    let (container, node) = start().await;
    let mut session = None;
    for _ in 0..90 {
        match Session::connect_single_node(&node).await {
            Ok(s) => {
                session = Some(s);
                break;
            }
            Err(_) => tokio::time::sleep(Duration::from_secs(2)).await,
        }
    }
    let session = session.expect("scylla did not come up");
    session.create_table::<UserX>().await.unwrap();

    eprintln!("loading {records} records ...");
    let t = Instant::now();
    let sem = Arc::new(tokio::sync::Semaphore::new(128));
    let mut set = tokio::task::JoinSet::new();
    for i in 0..records {
        let (s, sem) = (session.clone(), sem.clone());
        set.spawn(async move {
            let _p = sem.acquire().await.unwrap();
            s.insert(&userx(format!("user{}", i + 1), i)).await.unwrap();
        });
    }
    while set.join_next().await.is_some() {}
    eprintln!("loaded in {:.1}s", t.elapsed().as_secs_f64());

    // contenders
    let raw_sel = session.driver().prepare("SELECT ycsb_key, field0, field1, field2, field3, field4, field5, field6, field7, field8, field9 FROM ycsb.usertable WHERE ycsb_key = ?").await.unwrap();
    let raw_ins = session.driver().prepare("INSERT INTO ycsb.usertable (ycsb_key, field0, field1, field2, field3, field4, field5, field6, field7, field8, field9) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)").await.unwrap();
    let charybdis_session = {
        let inner = Session::connect_single_node(&node).await.unwrap().try_into_driver().ok().unwrap();
        inner.use_keyspace("ycsb", false).await.unwrap();
        Arc::new(cqlx::scylla::client::caching_session::CachingSession::from(inner, 100))
    };
    let session = Arc::new(session);
    let raw_sel = Arc::new(raw_sel);
    let raw_ins = Arc::new(raw_ins);

    println!("workload,contender,ops_per_s,p50_us,p99_us,ops");
    for (wl, read_frac) in [("A (50/50)", 0.5), ("B (95/5)", 0.95), ("C (100/0)", 1.0)] {
        // cqlx
        let s1 = session.clone();
        let s2 = session.clone();
        let st = drive(
            tasks,
            secs,
            records,
            read_frac,
            Arc::new(move |k: String| {
                let s = s1.clone();
                async move {
                    let _ = UserX::query().ycsb_key(k).first(&s).await.unwrap();
                }
            }),
            Arc::new(move |k: String, seed: u64| {
                let s = s2.clone();
                async move {
                    s.insert(&userx(k, seed)).await.unwrap();
                }
            }),
        )
        .await;
        print_row(wl, "cqlx", &st);

        // raw driver
        let (a, b, c1, c2) = (session.clone(), session.clone(), raw_sel.clone(), raw_ins.clone());
        let st = drive(
            tasks,
            secs,
            records,
            read_frac,
            Arc::new(move |k: String| {
                let (s, p) = (a.clone(), c1.clone());
                async move {
                    let r = s.driver().execute_unpaged(&p, (k,)).await.unwrap();
                    let _ = r.into_rows_result().unwrap().rows::<RawRow>().unwrap().next();
                }
            }),
            Arc::new(move |k: String, seed: u64| {
                let (s, p) = (b.clone(), c2.clone());
                async move {
                    let u = userx(k, seed);
                    s.driver().execute_unpaged(&p, (u.ycsb_key, u.field0, u.field1, u.field2, u.field3, u.field4, u.field5, u.field6, u.field7, u.field8, u.field9)).await.unwrap();
                }
            }),
        )
        .await;
        print_row(wl, "raw scylla driver", &st);

        // charybdis
        let (d, e) = (charybdis_session.clone(), charybdis_session.clone());
        let st = drive(
            tasks,
            secs,
            records,
            read_frac,
            Arc::new(move |k: String| {
                let s = d.clone();
                async move {
                    let _ = UserC::find_by_primary_key_value((k,)).execute(&s).await.unwrap();
                }
            }),
            Arc::new(move |k: String, seed: u64| {
                let s = e.clone();
                async move {
                    userc(k, seed).insert().execute(&s).await.unwrap();
                }
            }),
        )
        .await;
        print_row(wl, "charybdis", &st);
    }
    drop(container);
}

fn print_row(wl: &str, name: &str, s: &Stats) {
    println!("{wl},{name},{:.0},{},{},{}", s.ops as f64 / s.secs, s.p50, s.p99, s.ops);
}
