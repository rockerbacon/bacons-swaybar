use std::{path::Path, ffi::{CString, CStr}, mem};

// taken from the C headers
const IN_NONBLOCK: i32 = 0x800;
const IN_ACCESS: i32 = 0x1;

const MAX_NAMELEN: usize = 64;
const Q_SIZE: usize = 64;

#[repr(C)]
pub struct InotifyMsg {
	pub wd: i32,
	pub mask: u32,
	pub cookie: u32,
	pub len: u32,
	name: [u8; MAX_NAMELEN],
}

const BUFSIZE: usize = mem::size_of::<InotifyMsg>() * Q_SIZE;

impl InotifyMsg {
	pub fn get_name(&self) -> &str {
		return CStr::from_bytes_until_nul(&self.name)
			.unwrap().to_str().unwrap();
	}
}

impl Default for InotifyMsg {
	fn default() -> InotifyMsg {
		return InotifyMsg {
			wd: Default::default(),
			mask: Default::default(),
			cookie: Default::default(),
			len: Default::default(),
			name: [Default::default(); MAX_NAMELEN],
		}
	}
}

extern {
	fn close(fd: i32) -> i32;
	fn inotify_init1(flags: i32) -> i32;
	fn inotify_add_watch(fd: i32, path: *const i8, mask: i32) -> i32;
}

#[link(name="sysfsinotify")]
extern {
	fn sysfsinotify_recvmsg(
		fd: i32,
		buf: *mut u8,
		buflen: usize,
		vec: *mut *const InotifyMsg,
		veclen: usize,
	) -> isize;
	fn sysfsinotify_discmsg(fd: i32, buf: *mut u8, buflen: usize) -> isize;
}

pub struct Inotify {
	buf: [u8; BUFSIZE],
	msgs: [*const InotifyMsg; Q_SIZE],
	fd: i32,
}

impl Inotify {
	pub fn new() -> Inotify {
		let fd = unsafe { inotify_init1(IN_NONBLOCK) };
		if fd < 0 {
			panic!("Failed to create watch descriptor");
		}

		return Inotify {
			buf: [Default::default(); BUFSIZE],
			msgs: [0 as *const InotifyMsg; Q_SIZE],
			fd,
		}
	}

	pub fn add_access_watch(&self, path: &Path) -> i32 {
		let c_path = CString::new(path.to_str().unwrap()).unwrap();

		let wd = unsafe {
			inotify_add_watch(self.fd, c_path.as_ptr(), IN_ACCESS)
		};

		if wd < 0 {
			panic!("Failed to add watch");
		}

		return wd;
	}

	pub fn recvmsg<'a>(&'a mut self) -> Vec<&'a InotifyMsg> {
		let msgcount: isize = unsafe {
			sysfsinotify_recvmsg(
				self.fd,
				&mut self.buf as *mut u8,
				BUFSIZE,
				&mut self.msgs as *mut *const InotifyMsg,
				Q_SIZE,
			)
		};

		match msgcount {
			-1 => panic!(
				"Failure reading msgs: {}",
				std::io::Error::last_os_error()
			),
			-2 => panic!("Message queue too small"),
			-3 => panic!("Inotify event name overflow"),
			_ => (),
		}

		let mut vec: Vec<&'a InotifyMsg> = Vec::new();
		for i in 0..msgcount {
			vec.push(unsafe { &*self.msgs[i as usize] });
		}

		return vec;
	}

	pub fn discmsg(&mut self) {
		let status = unsafe {
			sysfsinotify_discmsg(
				self.fd,
				&mut self.buf as *mut u8,
				BUFSIZE,
			)
		};
		if status < 0 {
			panic!("Failed to discard inotify messages: {}", std::io::Error::last_os_error());
		}
	}
}

impl Drop for Inotify {
	fn drop(&mut self) {
		if unsafe { close(self.fd) } < 0 {
			panic!("Failed to close inotify fd");
		}
	}
}
