use std::fmt;

use crate::common::icon;
use crate::common::Widget;
use crate::network::{interface, netlink};

const AF_INET: i32 = 2;

const SOCK_STREAM: i32 = 1;

extern {
	fn close(fd: i32) -> i32;
	fn socket(domain: i32, typ: i32, protocol: i32) -> i32;
}

#[derive(Clone,Copy)]
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

impl PartialEq for ConnStat {
	fn eq(&self, other: &Self) -> bool {
		return self.bitmap == other.bitmap;
	}
}
impl Eq for ConnStat {}

pub struct Network {
	sock: i32,
	nlsock: netlink::NlSock,
	ifaces: Vec<interface::Interface>,
	conn_stat: ConnStat,
}

impl Network {
	fn update_conn_stat(&mut self) -> bool {
		let prev_conn_stat = self.conn_stat;

		self.conn_stat.reset();
		for iface in &self.ifaces {
			match iface.class {
				interface::Class::Eth => {
					if iface.is_running() {
						self.conn_stat.set_wired();
					}
				},
				interface::Class::Wlan => {
					if iface.is_running() {
						self.conn_stat.set_wireless();
					}
				}
			}
		}

		return prev_conn_stat != self.conn_stat;
	}

	pub fn new() -> Network {
		let sock: i32 = unsafe { socket(AF_INET, SOCK_STREAM, 0) };
		if sock <= 0 {
			panic!("Could not open socket");
		}

		let mut net = Network {
			sock,
			nlsock: netlink::NlSock::new(),
			ifaces: interface::list(sock),
			conn_stat: ConnStat::new(),
		};
		net.update_conn_stat();

		return net;
	}
}

impl Widget for Network {
	fn update(&mut self) -> bool {
		let msgs = self.nlsock.recvmsg();

		if msgs.len() == 0 {
			return false;
		}

		for msg in &msgs {
			let mut i = 0;
			while i < self.ifaces.len() &&
				self.ifaces[i].index as u32 != msg.devidx
			{
				i += 1;
			}

			if i == self.ifaces.len() {
				// changed device is of no interest
				continue;
			}

			match msg.modop {
				netlink::IPADD => {
					self.ifaces[i].ipv4 = msg.ipv4;
				},
				netlink::IPRMV => {
					if self.ifaces[i].ipv4 != msg.ipv4 {
						panic!("IPv4 desync");
					}
					self.ifaces[i].ipv4 = 0;
				},
				_ => panic!("Invalid modop {}", msg.modop),
			}
		}

		return self.update_conn_stat();
	}
}

impl Drop for Network {
	fn drop(&mut self) {
		unsafe { close(self.sock) };
	}
}

impl fmt::Display for Network {
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
