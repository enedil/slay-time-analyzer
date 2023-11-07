mod telegram_utils;

use swayipc::{Connection, Event, EventType, Fallible, WindowEvent, WindowChange};
use rusqlite;

fn get_conn() -> Option<rusqlite::Connection> {
    if let Ok(conn) = rusqlite::Connection::open("windowevents.db") {
        Some(conn)
    } else {
        None
    }
}

fn handle_event(ev: WindowEvent) {
    match &ev.change {
        WindowChange::Focus | WindowChange::Title => {
            let table_name = match &ev.change {
                WindowChange::Focus => "focus",
                WindowChange::Title => "title",
                _ => unreachable!(),
            };
            if let Some(conn) = get_conn() {
                let sql = format!("INSERT INTO {}events VALUES(?1, ?2, CURRENT_TIMESTAMP)", table_name);

                let app_id = ev.container.app_id.clone().unwrap_or_default();
                let title = if app_id == "org.telegram.desktop" {
                    telegram_utils::strip_message_counts(ev.container.name.clone().unwrap_or_default())
                } else {
                    ev.container.name.clone().unwrap_or_default()
                };
                conn.execute(&sql, (&app_id, &title)).unwrap();
            }
        },
        _ => (),
    }
}

fn main() -> Fallible<()> {
    get_conn().map(|conn| {
        conn.execute("CREATE TABLE IF NOT EXISTS focusevents (app_id TEXT, title TEXT, timestamp TEXT)", ()).unwrap();
        conn.execute("CREATE TABLE IF NOT EXISTS titleevents (app_id TEXT, title TEXT, timestamp TEXT)", ()).unwrap();
    });

    for event in Connection::new()?.subscribe([EventType::Window])? {
        match event? {
            Event::Window(w) => handle_event(*w),
            _ => unreachable!(),
        }
    }
    Ok(())
}