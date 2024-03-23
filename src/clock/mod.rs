use std::cell::{Cell, RefCell};
use std::env;
use std::fmt;
use std::rc::Rc;

use crate::common::Widget;
use crate::common::Time;

#[derive(PartialEq)]
enum Precision {
	Minutes, Seconds
}

pub struct Clock {
	prec: Precision,
	time: Rc<RefCell<Time>>,
	next_update: Cell<i64>,
}

impl Clock {
	pub fn new(time: Rc<RefCell<Time>>) -> Clock {
		let precision = env::var("CLOCK_PRECISION")
			.unwrap_or(String::from("minutes"));

		let prec = match precision.as_str() {
			"minutes" => Precision::Minutes,
			"seconds" => Precision::Seconds,
			_ => panic!("Invalid CLOCK_PRECISION"),
		};

		let next_update = Cell::new(time.borrow().timestamp());
		return Clock {
			prec,
			time,
			next_update,
		}
	}
}

impl fmt::Display for Clock {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let t = self.time.borrow();
		write!(
			f,
			"{}-{:02}-{:02} {:02}:{:02}",
			t.year(),
			t.mon(),
			t.day(),
			t.hour(),
			t.min(),
		)?;

		if self.prec == Precision::Seconds {
			write!(f, ":{:02}", t.sec())?;
		}

		self.next_update.set(
			t.timestamp() + match self.prec {
				Precision::Minutes => 59i64 - t.sec() as i64,
				Precision::Seconds => 1i64,
			}
		);

		return Ok(());
	}
}

impl Widget for Clock {
	fn update(&mut self) -> bool {
		return self.time.borrow().timestamp() >= self.next_update.get();
	}

	fn on_click(&self) -> Option<&str> {
		return None;
	}
}
