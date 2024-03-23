use std::fmt;

pub trait Widget: fmt::Display {
	fn update(&mut self) -> bool;
	fn on_click(&self) -> Option<&str>;
}
