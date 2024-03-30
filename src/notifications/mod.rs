use std::fmt;
use std::fs;
use std::io::ErrorKind;
use std::net::Shutdown;
use std::os::unix::net::UnixDatagram;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use crate::common::icon;
use crate::common::Widget;

extern {
	fn geteuid() -> u32;
}

pub struct Notifications {
	buff: [u8; 1],
	enabled: bool,
	sock: UnixDatagram,
	sock_path: PathBuf,
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
		let uid = unsafe { geteuid() };
		let mut sock_path = PathBuf::from("/var/run/user");
		sock_path.push(uid.to_string());
		sock_path.push("notif-toggle.sock");

		let sock = UnixDatagram::bind(&sock_path).unwrap();
		sock.set_nonblocking(true).unwrap();

		return Notifications{
			buff: Default::default(),
			enabled: Notifications::get_notif_state(),
			sock,
			sock_path,
		};
	}
}

impl Drop for Notifications {
	fn drop(&mut self) {
		self.sock.shutdown(Shutdown::Both).unwrap();
		// FIXME this is not running
		fs::remove_file(&self.sock_path).unwrap();
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
