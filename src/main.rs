use std::env;

mod battery_status;
mod icon;
mod network_status;
mod time_status;
mod widget;

use widget::Widget;

fn main() {
	let mut time = time_status::new();
	let mut battery = battery_status::new();
	let mut network = network_status::new();

	let time_ptr = &mut time as *mut time_status::TimeStatus;

	let mut widgets: Vec<&mut dyn Widget> = vec![
		&mut battery,
		&mut network,
		&mut time,
	];

	let separator: String = env::var("SEPARATOR").unwrap_or(String::from("  ⋮  "));
	let suffix: String = env::var("SUFFIX").unwrap_or(String::from("  "));

	loop {
		for wid in widgets[..widgets.len()-1].iter() {
			print!("{}{}", wid, separator);
		}
		print!("{}", widgets[widgets.len()-1]);
		println!("{}", suffix);

		std::thread::sleep(unsafe {
			(*time_ptr).seconds_alignment_delay()
		});

		for wid in widgets.iter_mut() {
			wid.update();
		}
	}
}
