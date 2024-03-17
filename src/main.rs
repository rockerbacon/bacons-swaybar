mod common;

mod battery;
mod clock;
mod network;

use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use crate::common::Time;
use crate::common::Widget;

fn main() {
	let time = Rc::new(RefCell::new(Time::new()));

	let mut clock_wid = clock::Clock::new(time.clone());
	let mut battery_wid = battery::Battery::new();
	let mut network_wid = network::Network::new();

	let mut widgets: Vec<&mut dyn Widget> = vec![
		&mut battery_wid,
		&mut network_wid,
		&mut clock_wid,
	];

	println!("{{\"version\":1,\"click_events\":true}}");
	print!("[");
	loop {
		std::thread::sleep(
			Duration::new(0, time.borrow().align_ns() as u32)
		);

		let mut update = false;
		for wid in widgets.iter_mut() {
			update = wid.update() || update;
		}

		if update {
			print!(
				"[{{\"name\":\"0\",\"full_text\":\"   {}   \"}}",
				widgets[0],
			);
			for i in 1..widgets.len() {
				print!(
					",{{\"name\":\"{}\",\"full_text\":\"   {}   \"}}",
					i,
					widgets[i],
				);
			}
			println!("],");
		}
		time.borrow_mut().update();
	}
}
