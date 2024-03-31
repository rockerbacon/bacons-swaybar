use std::env;
use std::fmt;
use std::fs;
use std::io::ErrorKind;
use std::os::unix::net::UnixDatagram;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use crate::common::icon;
use crate::common::Widget;

pub struct Notifications {
	buff: [u8; 1],
	enabled: bool,
	sock: UnixDatagram,
}

impl Notifications {
	pub fn get_notif_state() -> bool {
		let mut cmd = Command::new("notifications-enabled");
		cmd.stdout(Stdio::null());
		cmd.stderr(Stdio::null());

		return match cmd.status() {
			Ok(status) => status.success(),
			Err(_) => false,
		};
	}

	pub fn new() -> Notifications {
		let mut sock_path = PathBuf::from(
			env::var("XDG_RUNTIME_DIR").unwrap_or(String::from("/tmp"))
		);
		sock_path.push("notif-toggle.sock");

		if sock_path.exists() {
			fs::remove_file(&sock_path).unwrap();
		}

		let sock = UnixDatagram::bind(&sock_path).unwrap();
		sock.set_nonblocking(true).unwrap();

		return Notifications{
			buff: Default::default(),
			enabled: Notifications::get_notif_state(),
			sock,
		};
	}
}

impl fmt::Display for Notifications {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		if self.enabled {
			write!(f, "{}", icon::BELL)?;
		} else {
			write!(f, "{}", icon::BELL_CROSS)?;
		}

		return Ok(());
	}
}

impl Widget for Notifications {
	fn update(&mut self) -> bool {
		match self.sock.recv(&mut self.buff) {
			Ok(bytes) => {
				if bytes > 0 {
					let prev = self.enabled;
					self.enabled = self.buff[0] == '1' as u8;
					return prev != self.enabled;
				}

				return false;
			},
			Err(e) if e.kind() == ErrorKind::WouldBlock => {
				return false;
			}
			Err(e) => panic!("{:?}", e),
		}
	}

	fn on_click(&self) -> Option<&str> {
		return Some("toggle-notifications");
	}
}
