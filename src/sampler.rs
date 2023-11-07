mod telegram_utils;
use log::{error, warn, info};
use rusqlite;
use std::time::SystemTime;
use std::{collections::HashMap, time::Duration};
use swayipc::{Connection, Node};

fn get_app_id(node: Node) -> Option<String> {
    if node.app_id.is_some() {
        return node.app_id;
    }
    match node.window_properties {
        Some(props) => props.class,
        None => None,
    }
}

fn get_active_telegram_node(node: Node) -> Option<Node> {
    node.find_focused(|x| get_app_id(x.clone()).is_some())
}

fn window_name_fixup(app_id: &String, name: String) -> String {
    if app_id.contains("telegram") { 
        telegram_utils::strip_message_counts(name)
    } else {
        name
    }
}

fn process(swayconn: &mut Connection) -> Option<(String, String)> {
    let tree = swayconn.get_tree();
    match tree {
        Ok(tree) => {
            if let Some(node) = get_active_telegram_node(tree) {
                let node_cp = node.clone();
                let app_id = get_app_id(node_cp).or_else(|| Some("none".into())).unwrap();

                match node.name {
                    Some(name) => {
                        let fixed_window_name = window_name_fixup(&app_id, name);
                        return Some((app_id, fixed_window_name));
                    }
                    None => {
                        warn!("Window is focused but doesn't have any window name.");
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
    counter: &HashMap<(String, String), usize>,
) {
    match ts.duration_since(std::time::UNIX_EPOCH) {
        Ok(d) => {
            for (key, value) in counter.iter() {
                match insert_statement.execute((&d.as_secs(), &d.subsec_nanos(), &key.0, &key.1, value)) {
                    Err(err) => {
                        error!("Could not record changes to {:?}, value={}, duration={:?}, err={}", key, value, d, err);
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
        .prepare("INSERT INTO sample VALUES(?1, ?2, ?3, ?4, ?5)")
        .unwrap();

    let mut conn = Connection::new().unwrap();

    let mut interval = tokio::time::interval(DURATION);

    let mut counter: HashMap<(String, String), usize> = HashMap::new();
    let mut ts = SystemTime::now();
    loop {
        interval.tick().await;
        let window = process(&mut conn);
        if let Some((appid, title)) = window {
            if counter.contains_key(&(appid.clone(), title.clone())) {
                *counter.get_mut(&(appid, title)).unwrap() += 1;
            } else {
                counter.insert((appid, title), 1);
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

    let mut q = db.prepare("SELECT appid, title, sum(count) FROM sample WHERE tv_sec >= ?1 GROUP BY appid, title").expect("couldn't prepare anlytics query");
    let mut res = q.query((&previous_ts.as_secs(), )).expect("couldn't execute sqlite analytics query");

    let mut results = vec![];
    while let Some(row) = res.next().expect("failed to read next row") {
        let appid: String = row.get(0).expect("could not read row");
        let title: String = row.get(1).expect("could not read row");
        let count: usize = row.get(2).expect("could not read row");
        results.push(((count as f64) * (DURATION.as_millis() as f64) / 1000.0, appid, title));
    };
    results.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
    results.reverse();

    println!("Chat | Time");
    for (time, appid, title) in results.iter() {
        println!("{time:10}s - {:50} {:50}", appid, title);
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    if let Some(_) = std::env::vars().find(|x| x.0 == "LOG_TERM") {
        env_logger::init();
    } else {
        systemd_journal_logger::JournalLog::new().unwrap().install().unwrap();
    }
    log::set_max_level(log::LevelFilter::Info);
    let args: Vec<_> = std::env::args().collect();

    let db = rusqlite::Connection::open("sampler.db").unwrap();

    db.execute("CREATE TABLE IF NOT EXISTS sample (tv_sec INTEGER, tv_nsec INTEGER, appid TEXT, title TEXT, count INTEGER)", ()).unwrap();


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
