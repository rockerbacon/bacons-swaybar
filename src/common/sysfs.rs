use inotify;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, ErrorKind};
use std::mem;
use std::path::PathBuf;
use std::str;
use std::str::FromStr;

const BUFSIZE: usize = 64;

struct ValBuf {
	bbuf: [u8; BUFSIZE],
	fbuf: [u8; BUFSIZE],
	end: usize,
}

impl ValBuf {
	pub fn get<T>(&self) -> T
	where T: FromStr, <T as FromStr>::Err: fmt::Debug
	{
		let val = T::from_str(
			str::from_utf8(
				&self.fbuf[0..self.end]
			).expect("Non utf8 attr value")
		).expect("Invalid attr value");

		return val;
	}

	pub fn changed(&self) -> bool {
		let diff: bool;
		let mut i: usize = 0;
		loop {
			match (self.bbuf[i] as char, self.fbuf[i] as char) {
				('\0' | '\n', '\0' | '\n') => {
					diff = false;
					break;
				},
				_ if self.bbuf[i] != self.fbuf[i] => {
					diff = true;
					break;
				},
				_ if i < BUFSIZE - 1 => i += 1,
				_ => {
					diff = false;
					break;
				}
			}
		}

		return diff;
	}

	pub fn flip_bufs(&mut self, bytes_read: usize) {
		let mut i = bytes_read;
		loop {
			i -= 1;
			match self.bbuf[i] as char {
				'\0' | '\n' | ' ' | '\t' => (),
				_ => break,
			}
		}

		mem::swap(&mut self.fbuf, &mut self.bbuf);
		self.end = i + 1;
	}
}

impl Display for ValBuf {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		return write!(
			f,
			"{}",
			str::from_utf8(&self.fbuf[0..self.end])
				.expect("Broken utf8"),
		);
	}
}

struct Attr {
	val: ValBuf,
	file: RefCell<File>,
}

struct Device {
	path: PathBuf,
	attrs: HashMap<String, Attr>,
}

pub struct Sysfs {
	root: PathBuf,
	watches: HashMap<String, inotify::WatchDescriptor>,
	devices: HashMap<inotify::WatchDescriptor, Device>,
	watcher: inotify::Inotify,
}

impl Sysfs {
	fn update_attr(attr: &mut Attr) -> bool {
		let mut file = attr.file.borrow_mut();
		file.seek(SeekFrom::Start(0)).expect("Failed to reset file");

		let bytes_read = file.read(&mut attr.val.bbuf)
			.expect("Could not read device attribute");

		if attr.val.changed() {
			attr.val.flip_bufs(bytes_read);
			return true;
		} else {
			return false;
		}
	}

	fn watch_attr(device: &mut Device, attr_name: &str) {
		let mut attr_path = device.path.clone();
		attr_path.push(attr_name);

		let file = RefCell::new(
			File::open(&attr_path).expect("Could not open device attribute")
		);

		let mut attr = Attr {
			file,
			val: ValBuf {
				bbuf: [0u8; BUFSIZE],
				fbuf: [0u8; BUFSIZE],
				end: 0,
			}
		};
		Sysfs::update_attr(&mut attr);

		device.attrs.insert(
			String::from(attr_name),
			attr,
		);
	}

	fn watch_dev(&mut self, name: &str, attr: &str) {
		let mut path = self.root.clone();
		path.push(name);

		let wd = self.watcher.watches()
			.add(&path, inotify::WatchMask::ACCESS)
			.expect("Failed to add watch for device");

		let mut device = Device{
			path,
			attrs: HashMap::new(),
		};
		Sysfs::watch_attr(&mut device, attr);

		self.watches.insert(
			String::from(name),
			wd.clone(),
		);
		self.devices.insert(wd, device);
	}

	/// Watches a new device attribute,
	/// returning the current attribute value
	pub fn watch(&mut self, device: &str, attr: &str) {
		match self.watches.get_mut(device) {
			Some(wd) => Sysfs::watch_attr(
				self.devices.get_mut(&wd).unwrap(),
				attr,
			),
			None => return self.watch_dev(device, attr),
		}
	}

	pub fn get<T>(&self, device: &str, attr: &str) -> T
	where T: FromStr, <T as FromStr>::Err: fmt::Debug
	{
		let wd = self.watches.get(device).expect("Unknown device");
		let attr = self.devices
			.get(&wd).unwrap()
			.attrs.get(attr).expect("Unknown attr");

		return attr.val.get::<T>();
	}

	/// Updates all watched device attributes
	/// Returns true if any updated occured and false otherwise
	pub fn update(&mut self) -> bool {
		let mut buffer = [0u8; 1024];

		let mut has_updates = false;
		match self.watcher.read_events(&mut buffer) {
			Ok(events) => {
				for e in events {
					match self.devices.get_mut(&e.wd) {
						Some(device) => {
							let attr_name: &str = e.name
								.unwrap_or_default().to_str()
								.expect("Broken inotify event name");

							match device.attrs.get_mut(attr_name) {
								Some(attr) => {
									let u = Sysfs::update_attr(attr);
									if u {
										println!("{:#?}, {}", e, attr.val);
									}
									has_updates = u || has_updates;
								},
								None => (),
							}
						},
						None => (),
					}
				}
			},
			Err(err) => {
				if err.kind() != ErrorKind::WouldBlock {
					panic!("Failure while reading inotify events");
				}
			}
		}

		if has_updates {
			// purge events created by Sysfs::update_attr()
			self.watcher.read_events(&mut buffer)
				.expect("Broken event purge");
		}

		return has_updates;
	}

	pub fn new(class: &str) -> Sysfs {
		let mut root: PathBuf = PathBuf::from("/sys/class");
		root.push(class);

		return Sysfs{
			root,
			watches: HashMap::new(),
			devices: HashMap::new(),
			watcher: inotify::Inotify::init().expect("Failed to init inotify"),
		};
	}

}
