#![feature(plugin, decl_macro)]
#![plugin(rocket_codegen)]

extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
extern crate chrono;

use rocket::State;
use rocket_contrib::{Json, JsonValue};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, RwLock};
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::{Duration, Instant};
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
struct RateLimit {
    start: usize,
    max: usize,
    restore_rate: usize,
    restore_timeout: usize,
    restore_at: String,
    key: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
struct RateLimitContainer {
    rate_limits: Vec<RateLimit>
}

struct ARateLimit {
    start: AtomicUsize,
    max: AtomicUsize,
    restore_rate: AtomicUsize,
    restore_timeout: AtomicUsize,
    restore_at: DateTime<FixedOffset>,
    key: Arc<String>
}

impl ARateLimit {
    fn new(key: String, start: usize, max: usize, restore_rate: usize, restore_timeout: usize, restore_at: String) -> ARateLimit {
        ARateLimit {
            start: AtomicUsize::new(start),
            max: AtomicUsize::new(max),
            restore_rate: AtomicUsize::new(restore_rate),
            restore_timeout: AtomicUsize::new(restore_timeout),
            restore_at: DateTime::parse_from_rfc3339(&restore_at).unwrap(),
            key: Arc::new(key)
        }
    }

    fn to_serializable(&self) -> RateLimit {
        RateLimit {
            start: self.start.load(Ordering::Relaxed),
            max: self.max.load(Ordering::Relaxed),
            restore_rate: self.restore_rate.load(Ordering::Relaxed),
            restore_timeout: self.restore_timeout.load(Ordering::Relaxed),
            restore_at: self.restore_at.to_rfc3339(),
            key: Some(self.key.to_string()),
        }
    }
}

struct RateLimitManager {
    rate_limits: RwLock<Vec<ARateLimit>>,
}

impl RateLimitManager {
    fn find_by_key(&self, key: &str) -> Vec<RateLimit> {
        let mut filtered_limits = Vec::new();

        self.rate_limits.read().unwrap().iter()
            .filter(|rl| *rl.key == key)
            .for_each(|rl| {
                filtered_limits.push(rl.to_serializable());
            });

        filtered_limits
    }
}

#[post("/<key>", data="<rate_limit>")]
fn new_rate_limit(key: String, rate_limit: Json<RateLimit>, rate_limit_manager: State<RateLimitManager>) -> &'static str {
    let new_limit = ARateLimit::new(key, rate_limit.start, rate_limit.max, rate_limit.restore_rate, rate_limit.restore_timeout, rate_limit.restore_at.clone());
    let mut limits = rate_limit_manager.rate_limits.write().unwrap();
    limits.push(new_limit);
    "Ok!"
}

#[get("/<key>")]
fn find_rate_limits(key: String, rate_limit_manager: State<RateLimitManager>) -> Json<RateLimitContainer> {
    Json(RateLimitContainer {
        rate_limits: rate_limit_manager.find_by_key(&key)
    })
}

fn main() {
    rocket::ignite()
        .manage(RateLimitManager { rate_limits: RwLock::new(vec![]) })
        .mount("/", routes![new_rate_limit, find_rate_limits])
        .launch();
}
