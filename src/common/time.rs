/*
 * Rust time features are severely lacking,
 * and external libraries are bloated.
 * So let's just use the standard C library.
 */

// defined in /usr/include/linux/time.h
const CLOCK_REALTIME: i32 = 0;

// see "man 3type tm" for more information
#[repr(C)]
struct CTime {
	pub tm_sec: i32,
	pub tm_min: i32,
	pub tm_hour: i32,
	pub tm_mday: i32,
	pub tm_mon: i32,
	pub tm_year: i32,
	pub tm_wday: i32,
	pub tm_yday: i32,
	pub tm_isdst: i32,
	pub tm_gmtoff: i64,
	pub tm_zone: *const u8,
}

// see "man 3type timespec" for more information
#[derive(Default)]
#[repr(C)]
struct CTimeSpec {
	tv_sec: i64,
	tv_nsec: i64,
}

extern {
	fn clock_gettime(clockid: i32, ts: *mut CTimeSpec) -> i32;
	fn localtime(ts: *const i64) -> *const CTime;
}

pub struct Time {
	ts: CTimeSpec,
	tm: *const CTime,
}

impl Time {
	pub fn sec(&self) -> i32 {
		return unsafe {
			(*self.tm).tm_sec
		};
	}

	pub fn min(&self) -> i32 {
		return unsafe {
			(*self.tm).tm_min
		};
	}

	pub fn hour(&self) -> i32 {
		return unsafe {
			(*self.tm).tm_hour
		};
	}

	pub fn day(&self) -> i32 {
		return unsafe {
			(*self.tm).tm_mday
		};
	}

	pub fn mon(&self) -> i32 {
		return unsafe {
			(*self.tm).tm_mon
		} + 1;
	}

	pub fn year(&self) -> i32 {
		return unsafe {
			1900 + (*self.tm).tm_year
		};
	}

	pub fn align_ns(&self) -> i64 {
		return 999_999_999 - self.ts.tv_nsec;
	}

	pub fn timestamp(&self) -> i64 {
		return self.ts.tv_sec;
	}

	pub fn update(&mut self) {
		unsafe {
			clock_gettime(CLOCK_REALTIME, &mut self.ts as *mut CTimeSpec)
		};
		self.tm = unsafe {
			localtime(&self.ts.tv_sec)
		};
	}

	pub fn new() -> Time {
		let mut t = Time {
			ts: Default::default(),
			tm: 0 as *const CTime,
		};
		t.update();
		return t;
	}
}
