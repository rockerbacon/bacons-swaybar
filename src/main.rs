use std::env;

mod battery_status;
mod icon;
mod network_status;
mod time_status;
mod widget;

use widget::Widget;

use std::cell::RefCell;

fn main() {
	let time = RefCell::new(time_status::new());
	let battery = RefCell::new(battery_status::new());
	let network = RefCell::new(network_status::new());

	let widgets: Vec<&RefCell<dyn Widget>> = vec![
		&battery,
		&network,
		&time,
	];

	let separator: String = env::var("SEPARATOR").unwrap_or(String::from("  ⋮  "));
	let suffix: String = env::var("SUFFIX").unwrap_or(String::from("  "));

	loop {
		for wid in widgets[..widgets.len()-1].iter() {
			print!("{}{}", wid.borrow(), separator);
		}
		print!("{}", widgets[widgets.len()-1].borrow());
		println!("{}", suffix);

		std::thread::sleep(time.borrow_mut().seconds_alignment_delay());

		for wid in widgets.iter() {
			wid.borrow_mut().update();
		}
	}
}
