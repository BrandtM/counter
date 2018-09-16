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
use std::sync::{Arc, RwLock, Mutex};
use std::fs;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::time::{Duration, Instant};
use chrono::prelude::*;

mod rate_limit;
use rate_limit::*;

#[post("/<key>", data="<rate_limit>")]
fn new_rate_limit(key: String, rate_limit: Json<RateLimit>, rate_limit_container: State<RateLimitContainer>) -> &'static str {
    let new_limit = RateLimit::from_json(Some(key), rate_limit);
    let mut limits = rate_limit_container.rate_limits.lock().unwrap();
    limits.push(new_limit);
    "Ok!"
}

#[get("/<key>")]
fn find_rate_limits(key: String, rate_limit_container: State<RateLimitContainer>) -> Json<Vec<UnsafeRateLimit>> {
    Json(rate_limit_container.find_by_key(&key))
}

fn main() {
    rocket::ignite()
        .manage(RateLimitContainer { rate_limits: Arc::new(Mutex::new(vec![])) })
        .mount("/", routes![new_rate_limit, find_rate_limits])
        .launch();
}
