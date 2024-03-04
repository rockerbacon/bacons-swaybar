use std::env;

mod battery_status;
mod network_status;
mod time_status;

fn main() {
	let mut battery: battery_status::BatteryStatus = battery_status::new();
	let mut time: time_status::TimeStatus = time_status::new();
	let mut net: network_status::NetworkStatus = network_status::new();
	let separator: String = env::var("SEPARATOR").unwrap_or(String::from("  ⋮  "));
	let suffix: String = env::var("SUFFIX").unwrap_or(String::from("  "));

	loop {
		print!("{}{}", battery.to_string(), separator);
		print!("{}{}", net.to_string(), separator);
		print!("{}", time.to_string());
		println!("{}", suffix);

		std::thread::sleep(time.seconds_alignment_delay());

		battery.update();
		net.update();
		time.update();
	}
}
