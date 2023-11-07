mod telegram_utils;
use log::{error, trace, warn, info};
use rusqlite;
use std::time::SystemTime;
use std::{collections::HashMap, time::Duration};
use swayipc::{Connection, Node};

fn get_active_telegram_node(node: Node) -> Option<Node> {
    node.find_focused(|x| x.app_id == Some("org.telegram.desktop".into()))
}

fn process(swayconn: &mut Connection) -> Option<String> {
    let tree = swayconn.get_tree();
    match tree {
        Ok(tree) => {
            if let Some(node) = get_active_telegram_node(tree) {
                match node.name {
                    Some(name) => {
                        trace!("{:?} tg_chat={}", SystemTime::now(), name);
                        return Some(telegram_utils::strip_message_counts(name));
                    }
                    None => {
                        warn!("Telegram window is focused but doesn't have any window name.");
                    }
                }
            }
        }
        Err(e) => {
            error!("Could not get tree! {}", e);
        }
    };
    None
}

fn record_stats(
    ts: SystemTime,
    insert_statement: &mut rusqlite::Statement,
    counter: &HashMap<String, usize>,
) {
    match ts.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => {
            for (key, value) in counter.iter() {
                match insert_statement.execute((&d.as_secs(), &d.subsec_nanos(), key, value)) {
                    Err(err) => {
                        error!("Could not record changes to {}, value={}, duration={:?}, err={}", key, value, d, err);
                    },
                    _ => ()
                }
            }
        },
        Err(err) => {
            error!("Detected time drift from UNIX_EPOCH! {}", err);
        }
    }
}

const DURATION: Duration = Duration::from_millis(10);
const SAMPLE_WINDOW_SIZE: u64 = 5;

async fn sampler(db: rusqlite::Connection) {
    let mut insert_statement = db
        .prepare("INSERT INTO sample VALUES(?1, ?2, ?3, ?4)")
        .unwrap();

    let mut conn = Connection::new().unwrap();

    let mut interval = tokio::time::interval(DURATION);

    let mut counter: HashMap<String, usize> = HashMap::new();
    let mut ts = SystemTime::now();
    loop {
        interval.tick().await;
        let window = process(&mut conn);
        if let Some(chat_name) = window {
            if counter.contains_key(&chat_name) {
                *counter.get_mut(&chat_name).unwrap() += 1;
            } else {
                counter.insert(chat_name, 1);
            }
        }

        match ts.elapsed() {
            Ok(duration) => {
                if duration.as_secs() >= SAMPLE_WINDOW_SIZE {
                    record_stats(ts, &mut insert_statement, &counter);
                    ts = SystemTime::now();
                    counter.clear();
                }
            }
            Err(err) => {
                error!("Detected time drift! {}", err);
            }
        }
    }
}

fn analyze(db: rusqlite::Connection) {
    let now = SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap();
    let previous_ts = now.saturating_sub(Duration::from_secs(60*60*24));

    let mut q = db.prepare("SELECT chat_name, sum(count) FROM sample WHERE tv_sec >= ?1 GROUP BY chat_name").expect("couldn't prepare anlytics query");
    let mut res = q.query((&previous_ts.as_secs(), )).expect("couldn't execute sqlite analytics query");

    let mut results = vec![];
    while let Some(row) = res.next().expect("failed to read next row") {
        let chat_name: String = row.get(0).expect("could not read row");
        let count: usize = row.get(1).expect("could not read row");
        results.push(((count as f64) * (DURATION.as_millis() as f64) / 1000.0, chat_name));
    };
    results.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());

    println!("Chat | Time");
    for (time, chat_name) in results.iter() {
        println!("{:50} - {}s", chat_name, time);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    systemd_journal_logger::JournalLog::new().unwrap().install().unwrap();
    log::set_max_level(log::LevelFilter::Info);
    // env_logger::init();
    let args: Vec<_> = std::env::args().collect();

    let db = rusqlite::Connection::open("sampler.db").unwrap();

    db.execute("CREATE TABLE IF NOT EXISTS sample (tv_sec INTEGER, tv_nsec INTEGER, chat_name TEXT, count INTEGER)", ()).unwrap();


    match args[1].as_str() {
        "sample" => {
            info!("Started collecting Telegram stats");
            sampler(db).await;
        },
        "analyze" => {
            analyze(db);
        }
        _ => {
            println!("Valid subcommands: sample, analyze");
        }
    };
}
