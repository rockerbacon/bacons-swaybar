use std::mem;
use std::collections::HashSet;
use std::ffi::CStr;

#[derive(Clone, Copy)]
pub enum Class { Eth, Wlan }

pub struct Interface {
	buffer: libc::ifreq,
	class: Class,
	running: bool,
}

impl Interface {
	pub fn get_class(&self) -> Class {
		return self.class;
	}

	pub fn is_running(&self) -> bool {
		return self.running;
	}

	fn new_buffer(device: &String) -> libc::ifreq {
		if device.len() > libc::IFNAMSIZ {
			panic!("Bad ifdevice name");
		}

		let mut buf: mem::MaybeUninit<libc::ifreq> = mem::MaybeUninit::zeroed();
		let name_bytes = device.as_bytes();
		unsafe {
			let name_buffer: &mut [i8; libc::IFNAMSIZ] = &mut(*buf.as_mut_ptr()).ifr_name;

			for i in 0..name_bytes.len() {
				name_buffer[i] = name_bytes[i] as i8;
			}
			name_buffer[name_bytes.len()] = 0;
		}

		unsafe { return buf.assume_init() };
	}

	unsafe fn query_class(buffer: &mut libc::ifreq, sock: i32) -> Class {
		if libc::ioctl(sock, libc::SIOCGIWNAME, buffer as *mut libc::ifreq) == 0 {
			return Class::Wlan;
		} else {
			return Class::Eth;
		}
	}

	unsafe fn query_is_running(buffer: &mut libc::ifreq, sock: i32) -> bool {
		return libc::ioctl(sock, libc::SIOCGIFADDR, buffer as *mut libc::ifreq) == 0;
	}

	pub fn can_display(flags: i32) -> bool {
		return (libc::IFF_LOOPBACK | libc::IFF_NOARP) & flags == 0;
	}

	pub fn new(name: String, sock: i32) -> Interface {
		let mut buffer = Interface::new_buffer(&name);

		return Interface {
			buffer,
			class: unsafe { Interface::query_class(&mut buffer, sock) },
			running: unsafe { Interface::query_is_running(&mut buffer, sock) },
		};
	}

	pub fn update(&mut self, sock: i32) {
		self.running = unsafe { Interface::query_is_running(&mut self.buffer, sock) };
	}
}

pub fn list(sock: i32) -> Vec<Interface> {
	let mut addrs_init: mem::MaybeUninit<*mut libc::ifaddrs> = mem::MaybeUninit::uninit();
	let addrs_ptr: *mut *mut libc::ifaddrs = addrs_init.as_mut_ptr();
	if unsafe { libc::getifaddrs(addrs_ptr) } != 0 {
		panic!("Could not get interface addresses");
	}
	let addrs: *mut libc::ifaddrs = unsafe { *addrs_ptr };

	let mut known_ifaces: HashSet<String> = HashSet::new();
	let mut ifaces: Vec<Interface> = Vec::new();
	let mut it: *mut libc::ifaddrs = addrs;
	while !it.is_null() {
		let addr: &libc::ifaddrs = unsafe { &*it };

		let flags: i32 = addr.ifa_flags as i32;
		if Interface::can_display(flags) {
			let name: String = unsafe {
				String::from(
					CStr::from_ptr(addr.ifa_name).to_str().expect("Broken iface name")
				)
			};

			if !known_ifaces.contains(&name) {
				known_ifaces.insert(name.clone());
				ifaces.push(Interface::new(name, sock));
			}
		}

		it = addr.ifa_next;
	}

	unsafe {
		libc::freeifaddrs(addrs);
	}

	return ifaces;
}
