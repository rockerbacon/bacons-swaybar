use std::env;
use std::fmt;

use crate::common::icon;
use crate::common::Widget;
use crate::common::Sysfs;

pub struct Battery {
	anim_cycle: u8,
	bar: u8,
	max_charge: u8,
	sysfs: Sysfs,
	plugged_in: bool,
}

impl Battery {
	fn get_pct(&self) -> u8 {
		return self.sysfs.get::<u8>("BAT0", "capacity");
	}

	fn update_plugged_in(&mut self) {
		self.plugged_in = self.sysfs.get::<char>("AC", "online") == '1';
	}

	fn update_bar(&mut self, pct: u8) {
		match pct {
			(0..=15) => self.bar = 0,
			(16..=33) => self.bar = 0b001,
			(34..=65) => self.bar = 0b011,
			(66..=100) => self.bar = 0b111,
			_ => (),
		};

		self.anim_cycle = 0;
	}

	fn animate_bar(&mut self) {
		if self.anim_cycle == 0 {
			self.bar >>= 1;
		} else {
			self.bar = (self.bar << 1) + 1;
		}

		self.anim_cycle ^= 1;
	}

	fn bar_icon(&self, bit: u8) -> char {
		if (bit & self.bar) != 0 {
			return icon::SQR_L;
		}

		return icon::SQR_S;
	}

	pub fn new() -> Battery {
		let max_charge = match env::var("MAX_CHARGE") {
			Ok(val) => val.parse::<u8>().expect("invalid MAX_CHARGE"),
			Err(_) => 100,
		};

		let mut bat = Battery {
			anim_cycle: 0,
			bar: 0,
			max_charge,
			sysfs: Sysfs::new("power_supply"),
			plugged_in: false,
		};

		bat.sysfs.watch("BAT0", "capacity");
		bat.sysfs.watch("AC", "online");

		bat.update_plugged_in();
		bat.update_bar(bat.get_pct());

		return bat;
	}
}

impl fmt::Display for Battery {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let main_icn = match (self.plugged_in, self.bar) {
			(true, ..) => icon::BOLT,
			(false, 0b1..=0b111) => icon::BATT_HIG,
			_ => icon::BATT_LOW,
		};

		return write!(f, "{} {}{}{}", main_icn, self.bar_icon(0b1), self.bar_icon(0b10), self.bar_icon(0b100));
	}
}

impl Widget for Battery {
	fn update(&mut self) -> bool {
		let prev_bar = self.bar;
		let prev_plugged_in = self.plugged_in;

		let has_updates = self.sysfs.update();
		let pct = self.get_pct();

		if has_updates {
			self.update_plugged_in();
			self.update_bar(pct);
		} 

		if self.plugged_in && pct < self.max_charge - 5 {
			self.animate_bar();
		}

		return prev_bar != self.bar ||
			prev_plugged_in != self.plugged_in;
	}

	fn on_click(&self) -> Option<&str> {
		return Some("display-battery-stats");
	}
}
