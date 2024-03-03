mod battery_status;
mod time_status;

fn main() {
	let mut battery: battery_status::BatteryStatus = battery_status::new();
	let mut time: time_status::TimeStatus = time_status::new();

	loop {
		println!(
			"{}    {}  ",
			battery.to_string(),
			time.to_string()
		);

		std::thread::sleep(time.seconds_alignment_delay());

		battery.update();
		time.update();
	}
}
