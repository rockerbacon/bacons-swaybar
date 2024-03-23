#[macro_use]
mod common;

mod battery;
mod clock;
mod network;

use std::cell::RefCell;
use std::io;
use std::io::Read;
use std::process::Command;
use std::process::Stdio;
use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::common::Time;
use crate::common::Widget;

fn skip_value(buffer: &[u8], offset: usize, bytecount: usize) -> usize {
	let mut i = offset;
	while i < bytecount {
		match buffer[i] as char {
			',' | '}' => break,
			_ => i += 1,
		}
	}
	i += 1;

	return i;
}

fn read_value(buffer: &[u8], offset: usize, bytecount: usize) -> (usize, String) {
	let mut i = offset;

	while i < bytecount && buffer[i] as char != ':' {
		i += 1;
	}
	i += 1;
	if i >= bytecount {
		panic!("No value");
	}

	while i < bytecount {
		match buffer[i] as char {
			' ' | '\t' => i += 1,
			_ => break,
		}
	}
	if i >= bytecount {
		panic!("Only empty spaces after paramater");
	}

	let mut val = String::new();
	while i < bytecount {
		match buffer[i] as char {
			',' | '}' => break,
			_ => val.push(buffer[i] as char),
		}
		i += 1;
	}
	i += 1;

	return (i, val);
}

fn read_param(buffer: &[u8], offset: usize, bytecount: usize) -> (usize, String) {
	let mut i = offset;
	while i < bytecount {
		match buffer[i] as char {
			'[' | ',' | ' ' | '\t' | '{' | '"' => i += 1,
			_ => break,
		}
	}
	if i >= bytecount {
		panic!("No more parameters");
	}

	let mut param = String::new();
	while i < bytecount && buffer[i] as char != '"' {
		param.push(buffer[i] as char);
		i += 1;
	}
	i += 1;
	if i >= bytecount {
		panic!("Nothing left after parameter");
	}

	return (i, param);
}

fn listen_for_clicks(actions: Vec<String>) {
	let mut buffer: [u8; 1024] = [0u8; 1024];
	let mut stdin = io::stdin();

	// skip useless beginning '[' line
	stdin.read(&mut buffer).unwrap();
	loop {
		let bytecount: usize = stdin.read(&mut buffer).unwrap();
		let mut i: usize = 0;
		let mut name: Option<String> = None;
		let mut button: u8 = 0;
		while i < bytecount && (name.is_none() || button == 0) {
			let (newoffset, param) = read_param(&buffer, i, bytecount);
			i = newoffset;
			match param.as_str() {
				"name" => {
					let (newoffset, val) = read_value(&buffer, i, bytecount);
					name = Some(val);
					i = newoffset;
				},
				"button" => {
					let (newoffset, val) = read_value(&buffer, i, bytecount);
					if val != "1" {
						i = bytecount;
					} else {
						i = newoffset;
						button = 1;
					}
				},
				_ => i = skip_value(&buffer, i, bytecount),
			}
		}

		match (name, button) {
			(Some(val), 1) => {
				let wid: usize = val[1..val.len()-1].parse::<usize>().unwrap();
				if actions[wid].len() > 0 {
					Command::new("swaymsg")
						.stdout(Stdio::null())
						.stderr(Stdio::null())
						.args(["exec", actions[wid].as_str()])
						.spawn().ok();
				}
			},
			_ => (),
		}
	}
}

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

	let mut actions: Vec<String> = Vec::new();
	for w in widgets.iter() {
		actions.push(match w.on_click() {
			Some(program) => String::from(program),
			None => String::new(),
		});
	}

	thread::spawn(|| { listen_for_clicks(actions) });

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
