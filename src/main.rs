use postgres::{Client, NoTls};
use chrono::{Utc, NaiveDateTime, DateTime};
use std::sync::{Arc, Mutex};
use std::thread::{spawn, sleep};
use std::time::Duration;


#[derive(Debug)]
struct SharedStorage {
    value: i32,
}

fn worker1() -> i32 {
    chrono::Utc::now().timestamp() as i32
}

fn worker2(conn: &mut Client, timestamp: i32) {
    // let mut conn = Client::connect("postgres://postgres:postgres@localhost:5432/clock_db", NoTls)
    //     .expect("Error while connecting to DB");    
    conn.execute("INSERT INTO data (timestamp) SELECT $1 WHERE NOT EXISTS (SELECT 1 FROM data WHERE timestamp = $1);", &[&timestamp]).expect("Error while inserting data");
}

fn human_time(timestamp: i32) -> String {
    let timestamp_as_i64 = timestamp as i64;
    let naive = NaiveDateTime::from_timestamp_opt(timestamp_as_i64,  0);
    let datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive.unwrap(), Utc);
    let formatted_time = datetime.format("%y.%m.%d %H:%M:%S").to_string();
    formatted_time
}

fn worker3(conn: &mut Client) -> String {
    // let mut conn = Client::connect("postgres://postgres:postgres@localhost:5432/clock_db", NoTls)
    //     .expect("Error while connecting to DB");
    let res = conn.query("SELECT timestamp FROM data ORDER BY id DESC LIMIT 1", &[])
        .expect("Error while selecting data");
    if res.is_empty() {
        "No data in DB".to_string()
    } else {
        let row = &res[0];
        let timestamp: i32 = row.get(0);
        human_time(timestamp)
    }
}

fn main() {
    let data = Arc::new(Mutex::new(SharedStorage { value: 0 }));
    let data_clone = data.clone();

    let thread1 = spawn(move || {
        sleep(Duration::from_secs(1));
        for i in 0..10 {
            let mut guard = data_clone.lock().unwrap();
            guard.value = worker1();
            // println!("T1 [{i}]: {} to the SharedStorage", guard.value);
            sleep(Duration::from_secs(1));
        }
    });

    let data_clone = data.clone();
    let thread2 = spawn(move || {
        let mut conn2 = Client::connect("postgres://postgres:postgres@localhost:5432/clock_db", NoTls)
            .expect("Error while connecting to DB");
        sleep(Duration::from_secs(1));
        for i in 0..10 {
            let guard = data_clone.lock().unwrap();
            let timestamp = guard.value;
            worker2(&mut conn2, timestamp);
            // println!("T2 [{i}]: From SharedStorage {} and to DB", guard.value);
            sleep(Duration::from_millis(500));
        }
    });

    let thread3 = spawn(move || {
        let mut conn3 = Client::connect("postgres://postgres:postgres@localhost:5432/clock_db", NoTls)
            .expect("Error while connecting to DB");
        sleep(Duration::from_secs(1));
        for _ in 0..10 {
            let formatted_time = worker3(&mut conn3);
            // println!("T3: {} from DB", formatted_time);
            println!(">>> {}", formatted_time);
            sleep(Duration::from_secs(1));
        }
    });

    thread1.join().unwrap();
    thread2.join().unwrap();
    thread3.join().unwrap();
}
