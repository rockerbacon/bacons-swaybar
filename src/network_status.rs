use libc;
use std::collections::HashSet;
use std::mem;
use std::ffi::CStr;
use std::fmt;
use super::icon;
use super::widget;

enum InterfaceClass { Eth, Wlan }

struct Interface {
	buffer: libc::ifreq,
	class: InterfaceClass,
	is_running: bool,
}

impl Interface {
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

	unsafe fn get_class(buffer: &mut libc::ifreq, sock: i32) -> InterfaceClass {
		if libc::ioctl(sock, libc::SIOCGIWNAME, buffer as *mut libc::ifreq) == 0 {
			return InterfaceClass::Wlan;
		} else {
			return InterfaceClass::Eth;
		}
	}

	unsafe fn is_running(buffer: &mut libc::ifreq, sock: i32) -> bool {
		return libc::ioctl(sock, libc::SIOCGIFADDR, buffer as *mut libc::ifreq) == 0;
	}

	pub fn can_display(flags: i32) -> bool {
		return (libc::IFF_LOOPBACK | libc::IFF_NOARP) & flags == 0;
	}

	pub fn new(name: String, sock: i32) -> Interface {
		let mut buffer = Interface::new_buffer(&name);

		return Interface {
			buffer,
			class: unsafe { Interface::get_class(&mut buffer, sock) },
			is_running: unsafe { Interface::is_running(&mut buffer, sock) },
		};
	}

	pub fn update(&mut self, sock: i32) {
		self.is_running = unsafe { Interface::is_running(&mut self.buffer, sock) };
	}
}

struct ConnStat {
	bitmap: u8,
}

impl ConnStat {
	pub fn new() -> ConnStat {
		return ConnStat { bitmap: 0 };
	}

	pub fn reset(&mut self) {
		self.bitmap = 0;
	}

	pub fn set_wired(&mut self) {
		self.bitmap |= 0b01;
	}

	pub fn set_wireless(&mut self) {
		self.bitmap |= 0b10;
	}

	pub fn is_wired(&self) -> bool {
		return self.bitmap & 0b01 != 0;
	}

	pub fn is_wireless(&self) -> bool {
		return self.bitmap & 0b10 != 0;
	}

	pub fn is_off(&self) -> bool {
		return self.bitmap == 0;
	}
}

pub struct NetworkStatus {
	sock: i32,
	eth_ifaces: Vec<Interface>,
	wlan_ifaces: Vec<Interface>,
	conn_stat: ConnStat,
}

impl widget::Widget for NetworkStatus {
	fn update(&mut self) {
		self.conn_stat.reset();

		for i in &mut self.eth_ifaces {
			i.update(self.sock);
			if i.is_running {
				self.conn_stat.set_wired();
			}
		}

		for i in &mut self.wlan_ifaces {
			i.update(self.sock);
			if i.is_running {
				self.conn_stat.set_wireless();
			}
		}
	}
}

impl Drop for NetworkStatus {
	fn drop(&mut self) {
		unsafe { libc::close(self.sock) };
	}
}

impl fmt::Display for NetworkStatus {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		write!(f, "{}", icon::LAPTOP)?;

		if self.conn_stat.is_wired() {
			write!(f, " - ")?;
		} else if self.conn_stat.is_wireless() {
			write!(f, "   ")?;
		} else {
			write!(f, " x ")?;
		}

		if self.conn_stat.is_off() {
			write!(f, "{}", icon::QUESTION)?;
		} else {
			write!(f, "{}", icon::GLOBE)?;
		}

		return Ok(());
	}
}

fn list_interfaces(sock: i32) -> Vec<Interface> {
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

pub fn new() -> NetworkStatus {
	let sock: i32 = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
	if sock <= 0 {
		panic!("Could not open socket");
	}

	let mut ifaces = list_interfaces(sock);
	let mut eth_ifaces: Vec<Interface> = Vec::new();
	let mut wlan_ifaces: Vec<Interface> = Vec::new();
	let mut conn_stat: ConnStat = ConnStat::new();

	for i in (0..ifaces.len()).rev() {
		let iface = &ifaces[i];
		match iface.class {
			InterfaceClass::Eth => {
				if iface.is_running {
					conn_stat.set_wired();
				}
				eth_ifaces.push(ifaces.swap_remove(i));
			},
			InterfaceClass::Wlan => {
				if iface.is_running {
					conn_stat.set_wireless();
				}
				wlan_ifaces.push(ifaces.swap_remove(i));
			}
		}
	}

	return NetworkStatus{
		sock,
		eth_ifaces,
		wlan_ifaces,
		conn_stat,
	};
}
