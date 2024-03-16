use std::io;

const Q_SIZE: usize = 4;

pub const IPADD: u8 = 20;
pub const IPRMV: u8 = 21;

#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct IfModMsg {
	pub devidx: u32,
	pub modop: u8, // enum (IPADD, IPRMV)
	pub ipv4: u32,
}

#[link(name="netlink")]
extern {
	fn nlsock_open() -> *mut u8;
	fn nlsock_close(ptr: *mut u8) -> i32;
	fn nlsock_recv(sock: *mut u8, vec: *mut IfModMsg, veclen: usize) -> i32;
}

pub struct NlSock {
	// Rust will not manipulate the C structure,
	// so abstract its implementation
	ptr: *mut u8,
	buf: [IfModMsg; Q_SIZE],
}

impl NlSock {
	pub fn recvmsg<'a>(&'a mut self) -> Vec<&'a IfModMsg> {
		let msgcount = unsafe {
			nlsock_recv(
				self.ptr,
				&mut self.buf as *mut IfModMsg,
				Q_SIZE
			)
		};

		if msgcount == -1 {
			panic!(
				"Failed to read netlink messages: {}",
				io::Error::last_os_error()
			);
		} else if msgcount < 0 {
			panic!("Failed to read netlink messages");
		}

		let mut vec: Vec<&'a IfModMsg> = Vec::new();
		for i in 0..msgcount {
			vec.push(&self.buf[i as usize]);
		}

		return vec;
	}

	pub fn new() -> NlSock {
		let ptr = unsafe { nlsock_open() };
		if ptr as usize == 0 {
			panic!("Failed to allocate netlink socket");
		}

		return NlSock {
			ptr,
			buf: Default::default(),
		}
	}
}

impl Drop for NlSock {
	fn drop(&mut self) {
		if unsafe { nlsock_close(self.ptr) } < 0 {
			panic!("Failed to deallocate socket");
		}
	}
}
