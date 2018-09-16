extern crate serde;
extern crate serde_json;

use rocket_contrib::{Json};
use std::sync::{Arc, Mutex};
use chrono::prelude::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct RateLimit {
	start: Arc<Mutex<usize>>,
	max: Arc<Mutex<usize>>,
	restore_rate: Arc<Mutex<usize>>,
	restore_timeout: Arc<Mutex<usize>>,
	restore_at: Arc<Mutex<DateTime<FixedOffset>>>,
	key: Arc<Mutex<Option<String>>>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UnsafeRateLimit {
	start: usize,
	max: usize,
	restore_rate: usize,
	restore_timeout: usize,
	restore_at: DateTime<FixedOffset>,
	key: Option<String>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RateLimitContainer {
	pub rate_limits:  Arc<Mutex<Vec<RateLimit>>>
}

impl RateLimit {
	pub fn from_json(key: Option<String>, rate_limit: Json<RateLimit>) -> RateLimit {
		RateLimit {
			key: Arc::new(Mutex::new(key)), 
			start: rate_limit.start.clone(), 
			max: rate_limit.max.clone(), 
			restore_rate: rate_limit.restore_rate.clone(), 
			restore_timeout: rate_limit.restore_timeout.clone(), 
			restore_at: rate_limit.restore_at.clone()
		}
	}

	pub fn get_unsafe_representation(&self) -> UnsafeRateLimit {
		let mut key: Option<String> = None;

		if let Some(ref k) = *self.key.lock().unwrap() {
			key = Some(String::from(&(*k.clone())));
		}

		UnsafeRateLimit {
			start: *self.start.lock().unwrap(),
			max: *self.max.lock().unwrap(),
			restore_rate: *self.restore_rate.lock().unwrap(),
			restore_timeout: *self.restore_timeout.lock().unwrap(),
			restore_at: *self.restore_at.lock().unwrap(),
			key
		}
	}
}

impl RateLimitContainer {
	pub fn find_by_key(&self, key: &str) -> Vec<UnsafeRateLimit> {
		let mut filtered_limits = Vec::new();

		self.rate_limits.lock().unwrap().iter()
			.filter(|rl| *rl.key.lock().unwrap() == Some(String::from(key)))
			.for_each(|rl| {
				filtered_limits.push(rl.get_unsafe_representation());
			});

		filtered_limits
	}
}
