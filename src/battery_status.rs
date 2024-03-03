use battery;
use battery::units::ratio::percent;

const BATTERY_CHARGING: char = '\u{1f50c}';
const BATTERY_LOW: char = '\u{1faab}';
const BATTERY_MID: char = '\u{1f50b}';
const BATTERY_HIG: char = '\u{1f50b}';

pub struct BatteryStatus {
	manager: battery::Manager,
	battery: battery::Battery,
	icon: char,
	pct: u8,
	last_displayed_icon: char,
	last_displayed_pct: u8,
}

impl BatteryStatus {
	pub fn update(&mut self) {
		self.manager.refresh(&mut self.battery).expect("Broken refresh");

		let state: battery::State = self.battery.state();
		let pct: f32 = self.battery.state_of_charge().get::<percent>();

		self.pct = pct as u8;

		if state == battery::State::Charging {
			self.icon = BATTERY_CHARGING;
		} else if self.pct > 66 {
			self.icon = BATTERY_HIG;
		} else if self.pct > 33 {
			self.icon = BATTERY_MID;
		} else {
			self.icon = BATTERY_LOW;
		}
	}

	pub fn to_string(&mut self) -> String {
		self.last_displayed_icon = self.icon;
		self.last_displayed_pct = self.pct;
		let mut output = String::new();
		output.push(self.icon);
		output.push_str(self.pct.to_string().as_str());
		output.push('%');
		return output;
	}

	pub fn changed(&self) -> bool {
		return self.icon != self.last_displayed_icon ||
			self.pct != self.last_displayed_pct;
	}
}

pub fn new() -> BatteryStatus {
	let manager: battery::Manager = battery::Manager::new().expect("Expected battery manager");
	let battery: battery::Battery = manager.batteries()
		.expect("Missing all batteries").next()
		.expect("Missing single battery").expect("Missing nested battery");

	let mut status: BatteryStatus = BatteryStatus {
		manager,
		battery,
		icon: '\0',
		pct: 0,
		last_displayed_icon: '\0',
		last_displayed_pct: 0,
	};

	status.update();
	return status;
}
