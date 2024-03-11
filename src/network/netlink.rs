use libc;

use std::cell::RefCell;
use std::io;
use std::mem;
use std::mem::MaybeUninit;
use std::process;

const BUFSIZE: usize = 1024;

// the following constants were taken from /usr/include/linux/netlink.h
const NLMSG_ALIGNTO: u32 = 4u32;

#[allow(non_snake_case)]
const fn NLMSG_ALIGN(len: u32) -> u32 {
	return ((len)+NLMSG_ALIGNTO-1) & !(NLMSG_ALIGNTO-1);
}

const NLMSG_HDRLEN: u32 = NLMSG_ALIGN(
	mem::size_of::<libc::nlmsghdr>() as u32
);

#[allow(non_snake_case)]
fn NLMSG_DATA<T>(nlh: *const libc::nlmsghdr) -> *const T {
	return (nlh as usize + NLMSG_HDRLEN as usize) as *const T;
}

#[allow(non_snake_case)]
fn NLMSG_NEXT(
	nlh: *const libc::nlmsghdr,
	len: isize
) -> (*const libc::nlmsghdr, isize) {
	let offset = unsafe { NLMSG_ALIGN((*nlh).nlmsg_len) };
	return (
		(nlh as usize + offset as usize) as *const libc::nlmsghdr,
		len - offset as isize,
	);
}

// taken from /usr/include/linux/rtnetlink.h
#[derive(Clone, Debug)]
#[repr(C)]
struct RtAttr {
	rta_len: u16,
	rta_type: u16,
}

// taken from /usr/include/linux/if_addr.h
#[derive(Clone, Debug)]
#[repr(C)]
struct IfAddrMsg {
	ifa_family: u8,
	ifa_prefixlen: u8,
	ifa_flags: u8,
	ifa_scope: u8,
	ifa_index: u32,
}

// the following constants were taken from /usr/include/linux/rtnetlink.h
const RTA_ALIGNTO: u16 = 4u16;

#[allow(non_snake_case)]
fn RTA_ALIGN (len: u16) -> u16 {
	return (len + RTA_ALIGNTO - 1) & !(RTA_ALIGNTO - 1);
}

const RTA_LENGTH: u16 = mem::size_of::<RtAttr>() as u16;

#[allow(non_snake_case)]
fn RTA_OK (attr: *const RtAttr, len: u16) -> bool { unsafe {
	return len >= RTA_LENGTH &&
		(*attr).rta_len >= RTA_LENGTH &&
		(*attr).rta_len <= len;
} }

#[allow(non_snake_case)]
fn RTA_NEXT (attr: *const RtAttr, attrlen: u16) -> (*const RtAttr, u16) {
	let offset = unsafe { RTA_ALIGN((*attr).rta_len) };
	return (
		(attr as usize + offset as usize) as *const RtAttr,
		attrlen - offset
	);
}

#[allow(non_snake_case)]
fn RTA_DATA (attr: *const RtAttr) -> *const libc::c_void {
	return (attr as usize + RTA_LENGTH as usize) as *const libc::c_void;
}

#[derive(Debug)]
enum Change {
	IpAdd, IpRmv,
}

#[derive(Debug)]
pub struct Msg {
	devidx: u32,
	change: Change,
	ips: Vec<u32>,
}

fn readipv4s (addr: *const IfAddrMsg) -> Vec<u32> { unsafe {
	let mut ipv4s: Vec<u32> = Vec::new();

	let mut it = (addr as usize + mem::size_of::<IfAddrMsg>()) as *const RtAttr;
	let mut unread: u16 = (*it).rta_len;

	while RTA_OK(it, unread) {
		match (*it).rta_type {
			libc::IFA_ADDRESS | libc::IFA_LOCAL => {
				let data_ptr: *const u32 = RTA_DATA(it) as *const u32;
				ipv4s.push(*data_ptr);
			},
			_ => (),
		}
		(it, unread) = RTA_NEXT(it, unread);
	}

	return ipv4s;
} }

fn readmsgs (bufv: *const libc::c_void, buflen: isize) -> Vec<Msg> { unsafe {
	let mut unread: isize = buflen;
	let mut it: *const libc::nlmsghdr = mem::transmute(bufv);

	let mut msgs: Vec<Msg> = Vec::new();
	loop {
		if (*it).nlmsg_type as i32 == libc::NLMSG_ERROR {
			// let err: *const libc::nlmsgerr = NLMSG_DATA(it)
			// 	as *const libc::nlmsgerr;
			panic!("netlink payload error");
		}

		let mut change: Change = Change::IpAdd;
		match (*it).nlmsg_type {
			libc::RTM_NEWADDR => change = Change::IpAdd,
			libc::RTM_DELADDR => change = Change::IpRmv,
			libc::RTM_GETADDR => {
				// FIXME don't duplicate code
				if (*it).nlmsg_flags as i32 & libc::NLM_F_MULTI != 0 {
					(it, unread) = NLMSG_NEXT(it, unread);
				} else {
					break;
				}
			},
			_ => panic!("Unexpected msg type {}", (*it).nlmsg_type),
		};

		let addr: *const IfAddrMsg = NLMSG_DATA(it);
		let ips: Vec<u32> = readipv4s(addr);
		msgs.push(Msg {
			devidx: (*addr).ifa_index,
			change,
			ips,
		});

		// FIXME don't duplicate code
		if (*it).nlmsg_flags as i32 & libc::NLM_F_MULTI != 0 {
			(it, unread) = NLMSG_NEXT(it, unread);
		} else {
			break;
		}
	}

	return msgs;
} }

pub struct Socket {
	fd: i32,
	bufh: RefCell<Box<libc::msghdr>>,

	// these are referenced within bufh as raw pointers
	// need to keep them in memory
	_addr: Box<libc::sockaddr_nl>,
	_buf: Box<[u8; BUFSIZE]>,
	bufv: Box<libc::iovec>,
}

impl Socket {
	pub fn new() -> Socket {
		let fd = unsafe {
			libc::socket(
				libc::AF_NETLINK,
				libc::SOCK_RAW,
				libc::NETLINK_ROUTE,
			)
		};

		if fd < 0 {
			panic!("Failed to open netlink socket");
		}

		let mut addr: Box<libc::sockaddr_nl> = Box::new(unsafe {
			MaybeUninit::zeroed().assume_init()
		});

		let mut buf = Box::new([0u8; BUFSIZE]);
		let mut bufv = Box::new(libc::iovec {
			iov_base: &mut *buf as *mut u8 as *mut libc::c_void,
			iov_len: BUFSIZE,
		});

		let mut bufh: Box<libc::msghdr> = Box::new(unsafe {
			MaybeUninit::zeroed().assume_init()
		});

		(*addr).nl_family = libc::AF_NETLINK as u16;
		(*addr).nl_groups = libc::RTMGRP_IPV4_IFADDR as u32;
		let tid = unsafe { libc::gettid() };
		(*addr).nl_pid = (tid as u32) << 16 | process::id();

		(*bufh).msg_name = &mut *addr
			as *mut libc::sockaddr_nl
			as *mut libc::c_void;
		(*bufh).msg_namelen =
			mem::size_of::<libc::sockaddr_nl>() as u32;
		(*bufh).msg_iov = &mut *bufv as *mut libc::iovec;
		(*bufh).msg_iovlen = 1;

		unsafe {
			let bres = libc::bind(
				fd,
				&*addr
					as *const libc::sockaddr_nl
					as *const libc::sockaddr,
				mem::size_of::<libc::sockaddr_nl>() as u32,
			);
			if bres < 0 {
				libc::close(fd);
				panic!("Failed to bind netlink socket");
			}

			let ores = libc::fcntl(fd, libc::F_SETFL, libc::O_NONBLOCK);
			if ores < 0 {
				libc::close(fd);
				panic!("Failed to set netlink socket flags");
			}
		}

		return Socket {
			fd,
			bufh: RefCell::new(bufh),
			_addr: addr,
			_buf: buf,
			bufv,
		}
	}

	pub fn recvmsg(&self) -> Vec<Msg> { unsafe {
		let buflen: isize = libc::recvmsg(
			self.fd,
			&mut **self.bufh.borrow_mut(),
			0
		);

		match buflen {
			1.. => return readmsgs((*self.bufv).iov_base, buflen),
			-1 => {
				let err = io::Error::last_os_error();
				if err.raw_os_error().unwrap() == libc::EWOULDBLOCK {
					return Vec::new();
				} else {
					panic!(
						"Failure receiving netlink msg {}",
						err
					);
				}
			},
			// this should never be reached
			_ => panic!("Unexpected recvmsg return value"),
		}
	} }
}

impl Drop for Socket {
	fn drop(&mut self) { unsafe {
		libc::close(self.fd);
	} }
}
