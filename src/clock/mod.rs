use chrono::TimeDelta;
use chrono::prelude::{DateTime, Local, Timelike};

use std::cell::Cell;
use std::env;
use std::fmt;
use std::time::Duration;

use crate::common::Widget;

const FORMAT_SECONDS: &str = "%Y-%m-%d %H:%M:%S";
const FORMAT_MINUTES: &str = "%Y-%m-%d %H:%M";

trait Alignment {
	fn next_update(&self, time: &DateTime<Local>) -> DateTime<Local>;
}

struct SecondsAlign {}
impl Alignment for SecondsAlign {
	fn next_update(&self, time: &DateTime<Local>) -> DateTime<Local> {
		return time.with_nanosecond(0).unwrap()
			+ TimeDelta::milliseconds(999);
	}
}
const SEC_ALIGN: &dyn Alignment = &SecondsAlign{};

struct MinutesAlign {}
impl Alignment for MinutesAlign {
	fn next_update(&self, time: &DateTime<Local>) -> DateTime<Local> {
		return time.with_second(0).unwrap().with_nanosecond(0).unwrap()
			+ TimeDelta::milliseconds(59999);
	}
}
const MIN_ALIGN: &dyn Alignment = &MinutesAlign{};

pub struct Clock {
	fmt: &'static str,
	time: DateTime<Local>,
	next_update: Cell<DateTime<Local>>,
	align: &'static dyn Alignment,
}

impl Clock {
	/// Calculates a delay that ensures the next time update
	/// has near perfect alignment with the start of the next second
	pub fn alignment_delay(&self) -> Duration {
		let target = self.time.with_nanosecond(0).unwrap()
			+ TimeDelta::seconds(1);

		return (target - self.time).to_std().expect("Broken time delta");
	}

	pub fn new() -> Clock {
		let precision = env::var("CLOCK_PRECISION")
			.unwrap_or(String::from("minutes"));

		let (fmt, align) = match precision.as_str() {
			"seconds" => (
				FORMAT_SECONDS,
				SEC_ALIGN,
			),
			"minutes" => (
				FORMAT_MINUTES,
				MIN_ALIGN,
			),
			_ => panic!("Invalid CLOCK_PRECISION"),
		};

		let now = Local::now();
		return Clock{
			fmt,
			time: now,
			next_update: Cell::new(now),
			align,
		}
	}
}

impl fmt::Display for Clock {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(
			f, "{}", self.time.format(self.fmt).to_string()
		)?;

		self.next_update.set(
			self.align.next_update(&self.time)
		);

		return Ok(());
	}
}

impl Widget for Clock {
	fn update(&mut self) -> bool {
		self.time = Local::now();
		return self.time >= self.next_update.get();
	}
}
