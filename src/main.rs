use std::time::Duration;

mod battery_status;
mod time_status;

fn main() {
	let mut battery: battery_status::BatteryStatus = battery_status::new();
	let mut time: time_status::TimeStatus = time_status::new();

	loop {
		if battery.changed() || time.changed() {
			println!(
				"{}    {}  ",
				battery.to_string(),
				time.to_string()
			);
		}

		std::thread::sleep(Duration::new(1, 0));

		battery.update();
		time.update();
	}
}
