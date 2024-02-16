// CREATE TABLE data (id SERIAL PRIMARY KEY, timestamp TIMESTAMP);
// INSERT INTO data (timestamp) SELECT $1 WHERE NOT EXISTS (SELECT 1 FROM data WHERE timestamp = $1);
// INSERT INTO data (timestamp) SELECT 1707999026 WHERE NOT EXISTS (SELECT 1 FROM data WHERE timestamp = 1707999026);
// SELECT timestamp FROM data ORDER BY id DESC LIMIT 1;

use postgres::{Client, NoTls};
use chrono::{Utc, NaiveDateTime, DateTime};
use std::sync::{Arc, Mutex};
use std::thread::{spawn, sleep};
use std::time::{Duration, SystemTime};

#[derive(Debug)]
struct SharedStorage {
    value: u64,
}

// fn worker1() -> u64 {
//     chrono::Utc::now().timestamp() as u64
// }

// fn worker2(conn: &mut Client, timestamp: u64) {
//     let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(timestamp);
//     conn.execute("INSERT INTO data (timestamp) SELECT $1 WHERE NOT EXISTS (SELECT 1 FROM data WHERE timestamp = $1);", &[&timestamp])
//         .expect("Error while inserting data");
// }

// fn human_time(timestamp: SystemTime) -> String {
//     let datetime: DateTime<Utc> = DateTime::from(timestamp);
//     let formatted_time = datetime.format("%y.%m.%d %H:%M:%S").to_string();
//     formatted_time
// }

// fn worker3(conn: &mut Client) -> String {
//     let res = conn.query("SELECT timestamp FROM data ORDER BY id DESC LIMIT 1", &[])
//         .expect("Error while selecting data");
//     if res.is_empty() {
//         "".to_string()
//     } else {
//         let row = &res[0];
//         let timestamp: SystemTime = row.get(0);
//         human_time(timestamp)
//     }
// }


fn main() {
    let data = Arc::new(Mutex::new(SharedStorage { value: 1707999026 }));
    
    let thread1 = spawn({
        let data_clone1 = data.clone();
        move || {
            while 1 == 1 {
                let mut guard = data_clone1.lock().unwrap();
                guard.value = chrono::Utc::now().timestamp() as u64;
                sleep(Duration::from_millis(1000));
            }
        }
    });

    
    let thread2 = spawn({
        

        let mut conn2 = Client::connect("postgres://postgres:postgres@localhost:5432/clock_db", NoTls)
                .expect("Error while connecting to DB");
        let data_clone2 = data.clone();
        move || {
            while 1 == 1 {
                
                // worker2(&mut conn2, data_clone2.lock().unwrap().value);
                let timestamp = SystemTime::UNIX_EPOCH + Duration::from_secs(data_clone2.lock().unwrap().value);
                conn2.execute("INSERT INTO data (timestamp) SELECT $1 WHERE NOT EXISTS (SELECT 1 FROM data WHERE timestamp = $1);", &[&timestamp])
                    .expect("Error while inserting data");

                sleep(Duration::from_millis(500));
            }
        }
    });

    let thread3 = spawn(move || {
        let mut conn3 = Client::connect("postgres://postgres:postgres@localhost:5432/clock_db", NoTls)
            .expect("Error while connecting to DB");

        while 1 == 1 {
            // let formatted_time = worker3(&mut conn3);
            let mut formatted_time: String = "".to_string();

            let res = conn3.query("SELECT timestamp FROM data ORDER BY id DESC LIMIT 1", &[])
                .expect("Error while selecting data");

            if res.is_empty() {
                formatted_time = "".to_string();
            } else {
                let row = &res[0];
                let timestamp: SystemTime = row.get(0);
                let datetime: DateTime<Utc> = DateTime::from(timestamp);
                formatted_time = datetime.format("%y.%m.%d %H:%M:%S").to_string();
                // human_time(timestamp)
            }

            if formatted_time != "" {
                println!("{:?}", formatted_time);
            }
            sleep(Duration::from_millis(1000));
        }
    });

    // thread1.join().expect("Error while joining threads");
    // thread2.join().expect("Error while joining threads");
    thread3.join().expect("Error while joining threads");
}
