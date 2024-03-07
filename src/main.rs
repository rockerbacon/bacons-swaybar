mod battery;
mod clock;
mod icon;
mod network;
mod widget;

use std::env;
use std::cell::RefCell;

use crate::widget::Widget;

fn main() {
	let clock_wid = RefCell::new(clock::Clock::new());
	let battery_wid = RefCell::new(battery::Battery::new());
	let network_wid = RefCell::new(network::Network::new());

	let widgets: Vec<&RefCell<dyn Widget>> = vec![
		&battery_wid,
		&network_wid,
		&clock_wid,
	];

	let separator: String = env::var("SEPARATOR").unwrap_or(String::from("  ⋮  "));
	let suffix: String = env::var("SUFFIX").unwrap_or(String::from("  "));

	loop {
		for wid in widgets[..widgets.len()-1].iter() {
			print!("{}{}", wid.borrow(), separator);
		}
		print!("{}", widgets[widgets.len()-1].borrow());
		println!("{}", suffix);

		std::thread::sleep(clock_wid.borrow_mut().seconds_alignment_delay());

		for wid in widgets.iter() {
			wid.borrow_mut().update();
		}
	}
}
