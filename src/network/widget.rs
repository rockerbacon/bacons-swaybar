use libc;

use std::fmt;

use crate::icon;
use crate::widget;
use crate::network::interface;

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

pub struct Network {
	sock: i32,
	eth_ifaces: Vec<interface::Interface>,
	wlan_ifaces: Vec<interface::Interface>,
	conn_stat: ConnStat,
}

impl Network {
	pub fn new() -> Network {
		let sock: i32 = unsafe { libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0) };
		if sock <= 0 {
			panic!("Could not open socket");
		}

		let mut ifaces = interface::list(sock);
		let mut eth_ifaces: Vec<interface::Interface> = Vec::new();
		let mut wlan_ifaces: Vec<interface::Interface> = Vec::new();
		let mut conn_stat: ConnStat = ConnStat::new();

		for i in (0..ifaces.len()).rev() {
			let iface = &ifaces[i];
			match iface.get_class() {
				interface::Class::Eth => {
					if iface.is_running() {
						conn_stat.set_wired();
					}
					eth_ifaces.push(ifaces.swap_remove(i));
				},
				interface::Class::Wlan => {
					if iface.is_running() {
						conn_stat.set_wireless();
					}
					wlan_ifaces.push(ifaces.swap_remove(i));
				}
			}
		}

		return Network{
			sock,
			eth_ifaces,
			wlan_ifaces,
			conn_stat,
		};
	}
}

impl widget::Widget for Network {
	fn update(&mut self) {
		self.conn_stat.reset();

		for i in &mut self.eth_ifaces {
			i.update(self.sock);
			if i.is_running() {
				self.conn_stat.set_wired();
			}
		}

		for i in &mut self.wlan_ifaces {
			i.update(self.sock);
			if i.is_running() {
				self.conn_stat.set_wireless();
			}
		}
	}
}

impl Drop for Network {
	fn drop(&mut self) {
		unsafe { libc::close(self.sock) };
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
