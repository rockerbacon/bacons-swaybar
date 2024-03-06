use std::fmt;

pub trait Widget: fmt::Display {
	fn update(&mut self);
}
