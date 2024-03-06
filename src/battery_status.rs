use battery;
use battery::State;
use battery::units::ratio::percent;
use std::cmp;
use std::env;
use crate::icon;
use std::fmt;
use super::widget;

const BATTERY_ANIM: [char; 2] = [icon::SQR_S, icon::SQR_L];

pub struct BatteryStatus {
	manager: battery::Manager,
	battery: battery::Battery,
	bar: [char; 3],
	pct: u8,
	max_charge: u8,
	clock: usize,
}

impl BatteryStatus {
	fn refresh_data_src(&mut self) {
		self.manager.refresh(&mut self.battery).expect("Broken refresh");
	}

	fn get_pct(&self) -> u8 {
		let raw: f32 = self.battery.state_of_charge().get::<percent>();
		let pct: f32 = raw / (self.max_charge as f32) * 100f32;

		return cmp::min(100, pct as u8);
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

	pub fn init(&mut self) {
		self.pct = self.get_pct();
		self.bar = match self.pct {
			90..=100 => [icon::SQR_L, icon::SQR_L, icon::SQR_L],
			66..=89 => [icon::SQR_L, icon::SQR_L, icon::SQR_S],
			33..=65 => [icon::SQR_L, icon::SQR_S, icon::SQR_S],
			_ => [icon::SQR_S, icon::SQR_S, icon::SQR_S],
		}
	}
}

impl fmt::Display for BatteryStatus {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		let main_icn = match (self.battery.state(), self.pct) {
			(State::Charging, ..) => icon::BOLT,
			(State::Unknown, 95..=100) => icon::BOLT,
			(.., 33..=100) => icon::BATT_HIG,
			_ => icon::BATT_LOW,
		};

		return write!(f, "{} {}{}{}", main_icn, self.bar[0], self.bar[1], self.bar[2]);
	}
}

impl widget::Widget for BatteryStatus {
	fn update(&mut self) {
		self.refresh_data_src();

		let state: battery::State = self.battery.state();
		self.pct = self.get_pct();

		match (state, self.pct) {
			(State::Charging, 90..=100) => self.charge_full(),
			(State::Charging, 66..=89) => self.charge_mid(2),
			(State::Charging, 33..=65) => self.charge_mid(1),
			(State::Charging, 0..=32) => self.charge_low(),
			(.., 66..=89) => self.discharge(2),
			(.., 33..=65) => self.discharge(1),
			(.., 0..=32) => self.discharge(0),
			_ => (),
		};
	}
}

pub fn new() -> BatteryStatus {
	let manager: battery::Manager = battery::Manager::new().expect("Expected battery manager");
	let battery: battery::Battery = manager.batteries()
		.expect("Missing all batteries").next()
		.expect("Missing single battery").expect("Missing nested battery");

	let max_charge = match env::var("MAX_CHARGE") {
		Ok(val) => val.parse::<u8>().expect("invalid MAX_CHARGE"),
		Err(_) => 100,
	};

	let mut status: BatteryStatus = BatteryStatus {
		manager,
		battery,
		bar: [icon::SQR_S, icon::SQR_S, icon::SQR_S],
		pct: 0,
		max_charge,
		clock: 0,
	};

	status.init();
	return status;
}
