use chrono::prelude::{DateTime, Local, Timelike};
use chrono::TimeDelta;
use std::time::Duration;
use std::fmt;
use super::widget;

pub struct Clock {
	now: DateTime<Local>,
}

impl Clock {
	/**
	 Calculates a delay that ensures the next time update
	 has near perfect alignment with the start of the next second
	 */
	pub fn seconds_alignment_delay(&mut self) -> Duration {
		let target = self.now.with_nanosecond(0).expect("Broken nanoseconds")
			+ TimeDelta::seconds(1);

		return (target - self.now).to_std().expect("Broken duration");
	}

	pub fn new() -> Clock {
		return Clock{
			now: Local::now(),
		}
	}
}

impl fmt::Display for Clock {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		return write!(
			f, "{}", self.now.format("%Y-%m-%d %H:%M:%S").to_string()
		);
	}
}

impl widget::Widget for Clock {
	fn update(&mut self) {
		self.now = Local::now();
	}
}