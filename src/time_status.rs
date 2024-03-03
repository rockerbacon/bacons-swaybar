use chrono::prelude::{DateTime, Local, Timelike};
use chrono::TimeDelta;

pub struct TimeStatus {
	now: DateTime<Local>,
	next_update: DateTime<Local>,
	is_display_outdated: bool,
}

impl TimeStatus {
	pub fn update(&mut self) {
		self.now = Local::now();
		if self.now >= self.next_update {
			self.next_update = self.now + TimeDelta::seconds(60 - self.now.second() as i64);
			self.is_display_outdated = true;
		}
	}

	pub fn to_string(&mut self) -> String {
		self.is_display_outdated = false;
		return self.now.format("%Y-%m-%d %H:%M").to_string();
	}

	pub fn changed(&self) -> bool {
		return self.is_display_outdated;
	}
}

pub fn new() -> TimeStatus {
	let mut time = TimeStatus {
		now: Local::now(),
		next_update: Local::now(),
		is_display_outdated: false,
	};

	time.update();
	return time;
}
