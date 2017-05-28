#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate rocket;

extern crate serde_json;
// #[macro_use]
// extern crate serde_derive;

use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use rocket::State;

extern crate hyper;
extern crate hyper_native_tls;

use std::io::Read;

use hyper::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;

extern crate json;

extern crate time;

extern crate iso_country;
use iso_country::Country;

struct Store(Arc<Mutex<Option<String>>>);

#[get("/")]
fn index(store: State<Store>) -> rocket::response::content::JSON<String> {
    let shared_data = {
        store
            .inner().0
            .lock()
            .unwrap()
            .clone()
    };

    let data: String = match shared_data {
        Some(data) => data,
        None => {
            println!("Serve error: Using fallback string for None shared data");

            "{}".to_string()
        }
    };

    rocket::response::content::JSON(
        data
    )
}

#[inline]
fn process_entry(data: &json::object::Object) -> json::object::Object {
    let mut out = json::object::Object::new();

    for (name, value) in data.iter() {
        if name == "e" {
            out["requests"] = value.clone();
        }

        if name == "n" {
            out["new"] = value.clone();
        }

        if name == "t" {
            out["total"] = value.clone();
        }

        if name == "u" {
            out["unique"] = value.clone();
        }

        if name == "d" {
            out["total_duration"] = value.clone();
        }

        if name == "ds" {
            // ignore
            continue;
        }
    }

    out
}

#[inline]
fn process_ctly(data: &json::JsonValue) -> json::JsonValue {
    let ts = time::now();

    let day = ts.tm_mday.to_string();
    let month = (ts.tm_mon + 1).to_string();
    let year = (ts.tm_year + 1900).to_string();

    let mut data = match data[year][month][day].clone() {
        json::JsonValue::Object(data) => data,
        _ => {
            return json::JsonValue::new_object();
        }
    };

    let mut out = json::object::Object::new();

    let mut per_country = json::object::Object::new();
    let mut intraday_hours = json::object::Object::new();

    for (name, value) in data.iter_mut() {
        let c: Result<i32, _> = name.parse();

        let value: json::object::Object = match *value {
            json::JsonValue::Object(ref mut data) => data.clone(),
            _ => {
                continue;
            }
        };

        if c.is_ok() {
            intraday_hours.insert(&name, json::JsonValue::Object(process_entry(&value)));

            continue;
        }

        // try country
        let c: Result<Country, _> = name.parse();

        if c.is_ok() {
            per_country.insert(&c.unwrap().name(), json::JsonValue::Object(process_entry(&value)));

            continue;
        }

    }

    out["total"] = json::JsonValue::Object(process_entry(&data));

    out["hours"] = json::JsonValue::Object(intraday_hours);
    out["countries"] = json::JsonValue::Object(per_country);

    json::JsonValue::Object(out)
}

fn main() {
    let api_token = match std::env::var("API_TOKEN") {
        Ok(token) => token,
        _ => {
            panic!("Could not find API token!");
        }
    };

    let endpoint_url = format!(
        "https://metrics.psychonautwiki.org/o?api_key={}&app_id=58277520195a624d00fdfaa8&method=users&action=refresh&_=1493844779775",

        api_token
    );

    let data: Option<String> = None;

    // mut
    let root_store = Arc::new(Mutex::new(data));

    /* rocket */

    let store = root_store.clone();

    thread::spawn(move || {
       rocket::ignite()
        .mount("/", routes![index])
        .manage(Store(store))
        .launch();
    });

    /* crawler */

    loop {
        let endpoint_url = endpoint_url.clone();
        let store = root_store.clone();

        let _ = thread::spawn(move || {
            let ssl = NativeTlsClient::new().unwrap();
            let connector = HttpsConnector::new(ssl);
            let client = Client::with_connector(connector);

            let mut res = client.get(&endpoint_url).send().expect("Connection error: Couldn't send request.");

            let mut buf = String::new();
            res.read_to_string(&mut buf).expect("Connection error: Couldn't read response.");

            let data: json::JsonValue = match json::parse(&buf) {
                Ok(data) => data,
                Err(err) => {
                    panic!("Parse error: {:?}", err);
                }
            };

            *store.lock().unwrap() = Some(process_ctly(&data).dump());

            thread::sleep(Duration::from_millis(2000));
        }).join();
    }
}