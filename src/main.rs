// CREATE DATABSE clock_db;
// CREATE TABLE data (id SERIAL PRIMARY KEY, timestamp TIMESTAMP);

use chrono::{DateTime, Utc};
use postgres::{Client, NoTls};
use std::sync::{Arc, Mutex};
use std::thread::{sleep, spawn};
use std::time::{Duration, SystemTime};

#[derive(Debug)]
struct SharedStorage {
    value: u64,
}

fn worker1() -> u64 {
    chrono::Utc::now().timestamp() as u64
}

fn worker2(conn: &mut Client, timestamp: u64) {
    let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp);
    conn.execute("INSERT INTO data (timestamp) SELECT $1 WHERE NOT EXISTS (SELECT 1 FROM data WHERE timestamp = $1);", &[&timestamp])
        .expect("Error while inserting data");
}

fn worker3(conn: &mut Client) -> String {
    let res = conn
        .query("SELECT timestamp FROM data ORDER BY id DESC LIMIT 1", &[])
        .expect("Error while selecting data");

    if res.is_empty() {
        String::from("")
    } else {
        let row = &res[0];
        let timestamp: SystemTime = row.get(0);
        let datetime: DateTime<Utc> = DateTime::from(timestamp);

        datetime.format("%y.%m.%d %H:%M:%S").to_string()
    }
}

fn main() {
    let data = Arc::new(Mutex::new(SharedStorage { value: 1707999026 }));

    let thread1 = spawn({
        let data_clone1 = data.clone();
        move || loop {
            {
                let mut guard = data_clone1.lock().unwrap();
                guard.value = worker1();
            }
            sleep(Duration::from_millis(1000));
        }
    });

    let thread2 = spawn({
        let mut conn2 = Client::connect(
            "postgres://postgres:postgres@localhost:5432/clock_db",
            NoTls,
        )
        .expect("Error while connecting to DB");
        let data_clone2 = data.clone();

        move || loop {
            let timestamp = data_clone2.lock().unwrap().value;
            worker2(&mut conn2, timestamp);
            sleep(Duration::from_millis(500));
        }
    });

    let thread3 = spawn(move || {
        let mut conn3 = Client::connect(
            "postgres://postgres:postgres@localhost:5432/clock_db",
            NoTls,
        )
        .expect("Error while connecting to DB");

        loop {
            let formatted_time = worker3(&mut conn3);

            if !formatted_time.is_empty() {
                println!("{:?}", formatted_time);
            }

            sleep(Duration::from_millis(1000));
        }
    });

    thread1.join().expect("Error while joining threads");
    thread2.join().expect("Error while joining threads");
    thread3.join().expect("Error while joining threads");
}
