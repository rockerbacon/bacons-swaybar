use std::mem;
use std::collections::HashSet;
use std::ffi::CStr;

#[derive(Clone, Copy)]
pub enum Class { Eth, Wlan }

pub struct Interface {
	index: u32,
	ipv4: u32,
	class: Class,
}

impl Interface {
	pub fn get_index(&self) -> u32 {
		return self.index;
	}

	pub fn get_class(&self) -> Class {
		return self.class;
	}

	pub fn set_ipv4(&mut self, ipv4: u32) {
		self.ipv4 = ipv4;
	}

	pub fn rm_ipv4(&mut self) {
		self.set_ipv4(0);
	}

	pub fn get_ipv4(&self) -> u32 {
		return self.ipv4;
	}

	pub fn is_running(&self) -> bool {
		return self.ipv4 != 0;
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

	fn query_class(buffer: &mut libc::ifreq, sock: i32) -> Class {
		if unsafe {
			libc::ioctl(sock, libc::SIOCGIWNAME, buffer as *mut libc::ifreq)
		} == 0 {
			return Class::Wlan;
		} else {
			return Class::Eth;
		}
	}

	fn query_ipv4(buffer: &mut libc::ifreq, sock: i32) -> u32 {
		if unsafe {
			libc::ioctl(sock, libc::SIOCGIFADDR, buffer as *mut libc::ifreq)
		} != 0 {
			// interface doesn't have an ipv4
			return 0;
		}

		let addr: &libc::sockaddr_in = unsafe { mem::transmute(&buffer.ifr_ifru.ifru_addr) };
		return addr.sin_addr.s_addr;
	}

	fn query_index(buffer: &mut libc::ifreq, sock: i32) -> u32 {
		if unsafe {
			libc::ioctl(sock, libc::SIOCGIFINDEX, buffer as *mut libc::ifreq)
		} != 0 {
			panic!("Failure querying device index");
		}

		return unsafe { buffer.ifr_ifru.ifru_ifindex as u32 };
	}

	pub fn can_display(flags: i32) -> bool {
		return (libc::IFF_LOOPBACK | libc::IFF_NOARP) & flags == 0;
	}

	pub fn new(name: String, sock: i32) -> Interface {
		let mut buffer = Interface::new_buffer(&name);

		return Interface {
			index: Interface::query_index(&mut buffer, sock),
			ipv4: Interface::query_ipv4(&mut buffer, sock),
			class: Interface::query_class(&mut buffer, sock),
		};
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
				ifaces.push(Interface::new(
					name,
					sock
				));
			}
		}

		it = addr.ifa_next;
	}

	unsafe {
		libc::freeifaddrs(addrs);
	}

	return ifaces;
}
