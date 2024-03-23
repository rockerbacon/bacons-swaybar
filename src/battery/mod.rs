use std::cmp;
use std::env;
use std::fmt;

use crate::common::icon;
use crate::common::Widget;
use crate::common::Sysfs;

const BATTERY_ANIM: [char; 2] = [icon::SQR_S, icon::SQR_L];

pub struct Battery {
	bar: [char; 3],
	pct: u8,
	max_charge: u8,
	clock: usize,
	sysfs: Sysfs,
	plugged_in: bool,
}

impl Battery {
	fn update_pct(&mut self) {
		let raw = self.sysfs.get::<u8>("BAT0", "capacity");
		let pct: f32 = raw as f32 / self.max_charge as f32 * 100f32;

		self.pct = cmp::min(100, pct as u8);
	}

	fn update_plugged_in(&mut self) {
		self.plugged_in = self.sysfs.get::<char>("AC", "online") == '1';
	}

	fn charge_low(&mut self) {
		self.bar[0] = BATTERY_ANIM[self.clock % 2];
		self.clock += 1;
	}

	fn charge_mid(&mut self, range: usize) {
		self.bar[range-1] = icon::SQR_L;
		self.bar[range] = BATTERY_ANIM[self.clock % 2];
		self.clock += 1;
	}

	fn charge_full(&mut self) {
		self.bar[2] = icon::SQR_L;
	}

	fn discharge(&mut self, level: usize) {
		self.bar[level] = icon::SQR_S;
	}

	pub fn new() -> Battery {
		let max_charge = match env::var("MAX_CHARGE") {
			Ok(val) => val.parse::<u8>().expect("invalid MAX_CHARGE"),
			Err(_) => 100,
		};

		let mut bat = Battery {
			bar: ['\0'; 3],
			pct: 0,
			max_charge,
			clock: 0,
			sysfs: Sysfs::new("power_supply"),
			plugged_in: false,
		};

		bat.sysfs.watch("BAT0", "capacity");
		bat.sysfs.watch("AC", "online");

		bat.update_pct();
		bat.update_plugged_in();

		bat.bar = match bat.pct {
			90..=100 => [icon::SQR_L, icon::SQR_L, icon::SQR_L],
			46..=89 => [icon::SQR_L, icon::SQR_L, icon::SQR_S],
			16..=45 => [icon::SQR_L, icon::SQR_S, icon::SQR_S],
			_ => [icon::SQR_S, icon::SQR_S, icon::SQR_S],
		};

		return bat;
	}
}

impl fmt::Display for Battery {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let main_icn = match (self.plugged_in, self.pct) {
			(true, ..) => icon::BOLT,
			(false, 16..=100) => icon::BATT_HIG,
			_ => icon::BATT_LOW,
		};

		return write!(f, "{} {}{}{}", main_icn, self.bar[0], self.bar[1], self.bar[2]);
	}
}

impl Widget for Battery {
	fn update(&mut self) -> bool {
		let has_updates = self.sysfs.update();

		if has_updates {
			self.update_plugged_in();
			self.update_pct();

			match (self.plugged_in, self.pct) {
				(true, 90..=100) => self.charge_full(),
				(true, 46..=89) => self.charge_mid(2),
				(true, 16..=45) => self.charge_mid(1),
				(true, 0..=15) => self.charge_low(),
				(false, 46..=89) => self.discharge(2),
				(false, 16..=45) => self.discharge(1),
				(false, 0..=15) => self.discharge(0),
				_ => (),
			};

			return true;
		} else if self.plugged_in {
			let is_animated = match self.pct {
				46..=89 => {
					self.charge_mid(2);
					return true;
				},
				16..=45 => {
					self.charge_mid(1);
					return true;
				},
				0..=15 => {
					self.charge_low();
					return true;
				},
				_ => false,
			};

			return is_animated;
		} else {
			return false;
        }
	}

	fn on_click(&self) -> Option<&str> {
		return Some("display-battery-stats");
	}
}
