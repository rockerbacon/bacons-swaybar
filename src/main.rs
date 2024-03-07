use std::env;

mod battery;
mod icon;
mod network_status;
mod time_status;
mod widget;

use widget::Widget;

use std::cell::RefCell;

fn main() {
	let time_wid = RefCell::new(time_status::new());
	let battery_wid = RefCell::new(battery::Battery::new());
	let network_wid = RefCell::new(network_status::new());

	let widgets: Vec<&RefCell<dyn Widget>> = vec![
		&battery_wid,
		&network_wid,
		&time_wid,
	];

	let separator: String = env::var("SEPARATOR").unwrap_or(String::from("  ⋮  "));
	let suffix: String = env::var("SUFFIX").unwrap_or(String::from("  "));

	loop {
		for wid in widgets[..widgets.len()-1].iter() {
			print!("{}{}", wid.borrow(), separator);
		}
		print!("{}", widgets[widgets.len()-1].borrow());
		println!("{}", suffix);

		std::thread::sleep(time_wid.borrow_mut().seconds_alignment_delay());

		for wid in widgets.iter() {
			wid.borrow_mut().update();
		}
	}
}
