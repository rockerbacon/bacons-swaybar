use battery;
use battery::State;
use battery::units::ratio::percent;
use std::cmp;
use std::env;

const BOLT: char = '\u{26a1}';
const SMALL_SQUARE: char = '\u{25ab}';
const LARGE_SQUARE: char = '\u{25a0}';
const BATTERY_ANIM: [char; 2] = [SMALL_SQUARE, LARGE_SQUARE];

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
		self.bar[range-1] = LARGE_SQUARE;
		self.bar[range] = BATTERY_ANIM[self.clock % 2];
		self.clock += 1;
	}

	fn charge_full(&mut self) {
		self.bar[2] = LARGE_SQUARE;
	}

	fn discharge(&mut self, level: usize) {
		self.bar[level] = SMALL_SQUARE;
	}

	pub fn update(&mut self) {
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

	pub fn init(&mut self) {
		self.pct = self.get_pct();
		self.bar = match self.pct {
			90..=100 => [LARGE_SQUARE, LARGE_SQUARE, LARGE_SQUARE],
			66..=89 => [LARGE_SQUARE, LARGE_SQUARE, SMALL_SQUARE],
			33..=65 => [LARGE_SQUARE, SMALL_SQUARE, SMALL_SQUARE],
			_ => [SMALL_SQUARE, SMALL_SQUARE, SMALL_SQUARE],
		}
	}

	pub fn to_string(&mut self) -> String {
		let mut output = String::new();
		output.push(BOLT);
		output.push(' ');
		output.push(self.bar[0]);
		output.push(self.bar[1]);
		output.push(self.bar[2]);
		return output;
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
		bar: [SMALL_SQUARE, SMALL_SQUARE, SMALL_SQUARE],
		pct: 0,
		max_charge,
		clock: 0,
	};

	status.init();
	return status;
}
