// TODO move all this C code out of Rust

use std::mem::{self, ManuallyDrop};
use std::collections::HashSet;
use std::ffi::CStr;

const IF_NAMESIZE: usize = 16;

const SIOCGIWNAME: u64 = 0x8B01;
const SIOCGIFADDR: u64 = 0x8915;
const SIOCGIFINDEX: u64 = 0x8933;

const IFF_LOOPBACK: u32 = 0x8;
const IFF_NOARP: u32 = 0x80;

#[repr(C)]
struct IfAddrs {
	ifa_next: *mut IfAddrs,
	ifa_name: *const i8,
	ifa_flags: u32,

	// these next three are defined very differently in the C header,
	// but the values are unused, so we can forego proper definitions
	ifa_addr: *const u8,
	ifa_netmask: *const u8,
	ifu_dstaddr: *const u8,

	ifa_data: *const u8,
}

#[repr(C)]
struct InAddr {
	s_addr: u32,
}

#[repr(C)]
struct SockAddrIn {
	sa_family: u16,
	sin_port: u16,
	sin_addr: InAddr,
}

// the C header defines more types,
// but these are the only ones we need
#[repr(C)]
union IfReqData {
	ifr_addr: ManuallyDrop<SockAddrIn>,
	ifr_ifindex: i32,
}

#[repr(C)]
struct IfReq {
	ifr_name: [i8; IF_NAMESIZE],
	data: IfReqData,
}

extern {
	fn ioctl(fd: i32, request: u64, argp: *mut IfReq) -> i32;
	fn getifaddrs(ifap: *mut *mut IfAddrs) -> i32;
	fn freeifaddrs(ifap: *mut IfAddrs);
}

#[derive(Clone, Copy)]
pub enum Class { Eth, Wlan }

pub struct Interface {
	pub index: i32,
	pub ipv4: u32,
	pub class: Class,
}

impl Interface {
	pub fn is_running(&self) -> bool {
		return self.ipv4 != 0;
	}

	fn new_buffer(device: &String) -> IfReq {
		if device.len() > IF_NAMESIZE {
			panic!("Bad ifdevice name");
		}

		let mut buf: mem::MaybeUninit<IfReq> = mem::MaybeUninit::zeroed();
		let name_bytes = device.as_bytes();
		unsafe {
			let name_buffer: &mut [i8; IF_NAMESIZE] = &mut(*buf.as_mut_ptr()).ifr_name;

			for i in 0..name_bytes.len() {
				name_buffer[i] = name_bytes[i] as i8;
			}
			name_buffer[name_bytes.len()] = 0;
		}

		unsafe { return buf.assume_init() };
	}

	fn query_class(buffer: &mut IfReq, sock: i32) -> Class {
		if unsafe {
			ioctl(sock, SIOCGIWNAME, buffer as *mut IfReq)
		} == 0 {
			return Class::Wlan;
		} else {
			return Class::Eth;
		}
	}

	fn query_ipv4(buffer: &mut IfReq, sock: i32) -> u32 {
		if unsafe {
			ioctl(sock, SIOCGIFADDR, buffer as *mut IfReq)
		} != 0 {
			// interface doesn't have an ipv4
			return 0;
		}

		let addr: &SockAddrIn = unsafe { mem::transmute(&buffer.data.ifr_addr) };
		return addr.sin_addr.s_addr;
	}

	fn query_index(buffer: &mut IfReq, sock: i32) -> i32 {
		if unsafe {
			ioctl(sock, SIOCGIFINDEX, buffer as *mut IfReq)
		} != 0 {
			panic!("Failure querying device index");
		}

		return unsafe { buffer.data.ifr_ifindex };
	}

	pub fn can_display(flags: u32) -> bool {
		return (IFF_LOOPBACK | IFF_NOARP) & flags == 0;
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
	let mut addrs_init: mem::MaybeUninit<*mut IfAddrs> = mem::MaybeUninit::uninit();
	let addrs_ptr: *mut *mut IfAddrs = addrs_init.as_mut_ptr();
	if unsafe { getifaddrs(addrs_ptr) } != 0 {
		panic!("Could not get interface addresses");
	}
	let addrs: *mut IfAddrs = unsafe { *addrs_ptr };

	let mut known_ifaces: HashSet<String> = HashSet::new();
	let mut ifaces: Vec<Interface> = Vec::new();
	let mut it: *mut IfAddrs = addrs;
	while !it.is_null() {
		let addr: &IfAddrs = unsafe { &*it };

		if Interface::can_display(addr.ifa_flags) {
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
		freeifaddrs(addrs);
	}

	return ifaces;
}
