use std::fmt;

use crate::common::icon;
use crate::common::Widget;

pub struct Notifications {}

impl Notifications {
	pub fn new() -> Notifications {
		return Notifications{};
	}
}

impl fmt::Display for Notifications {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		return write!(f, "{}", icon::BELL);
	}
}

impl Widget for Notifications {
	fn update(&mut self) -> bool {
		return false;
	}

	fn on_click(&self) -> Option<&str> {
		return None;
	}
}
