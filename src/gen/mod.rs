#[allow(dead_code)]

pub mod action;
pub use self::action::Action;

pub mod error;
pub use self::error::Error;

use std::slice;
use libc::{c_void, uint32_t};

use gpio::GPIO;

use generated::generator as gen_macros;

enum _Generator {}

extern "C" {
	fn gen_init(buffer_size: uint32_t, wave_size: uint32_t, delay: uint32_t) -> *mut _Generator;
	fn gen_free(gen: *mut _Generator);
	
	fn gen_run(gen: *mut _Generator, get_wave: extern fn(*mut uint32_t, *mut c_void), user_data: *mut c_void);
	fn gen_stop(gen: *mut _Generator);

	fn gen_position(gen: *const _Generator) -> uint32_t;
	fn gen_clear_position(gen: *mut _Generator);
}

pub struct GeneratorOptions {
	_buffer_size: u32,
	_wave_size: u32,
	_loop_delay: u32,
}

pub struct Generator {
	raw: *mut _Generator,
}

struct CallbackWrapper<'a> {
	callback: &'a mut FnMut() -> Action,
}

extern "C" fn gen_callback(raw_action: *mut uint32_t, userdata: *mut c_void) {
	unsafe {
		let ref mut get_action = (&mut *(userdata as *mut CallbackWrapper)).callback;
		let mut action = slice::from_raw_parts_mut(raw_action, gen_macros::ACTION_SIZE as usize);
		match get_action() {
			Action::None => {
				action[0] = gen_macros::ACTION_NONE;
			},
			Action::Wait { us } => {
				action[0] = gen_macros::ACTION_WAIT;
				action[1] = us as uint32_t;
			},
			Action::GPIO { on, off } => {
				action[0] = gen_macros::ACTION_GPIO;
				action[1] = on as uint32_t;
				action[2] = off as uint32_t;
			},
		}
	}
}

impl GeneratorOptions {
	pub fn new() -> Self {
		Self { 
			_buffer_size: 16,
			_wave_size: 256,
			_loop_delay: 10000
		}
	}

	pub fn buffer_size(mut self, size: u32) -> Self {
		self._buffer_size = size;
		self
	}

	pub fn wave_size(mut self, size: u32) -> Self {
		self._wave_size = size;
		self
	}

	pub fn loop_delay(mut self, delay: u32) -> Self {
		self._loop_delay = delay;
		self
	}

	pub fn build(self) -> Result<Generator, Error> {
		Generator::from_options(self)
	}
}

impl Generator {
	pub fn new() -> Result<Self, Error> {
		Self::from_options(GeneratorOptions::new())
	}

	pub fn from_options(opt: GeneratorOptions) -> Result<Self, Error> {
		let gen = unsafe { 
			gen_init(
				opt._buffer_size as uint32_t, 
				opt._wave_size as uint32_t,
				opt._loop_delay as uint32_t
			)
		};
		if !gen.is_null() {
			Ok(Self { raw: gen })
		} else {
			Err(Error::new())
		}
	}

	pub fn options() -> GeneratorOptions {
		GeneratorOptions::new()
	}

	#[allow(unused_variables)]
	pub fn run(&mut self, gpio: &mut GPIO, next_action: &mut FnMut() -> Action) {
		unsafe { 
			let mut wrapper = CallbackWrapper { callback: next_action };
			gen_run(self.raw, gen_callback, &mut wrapper as *mut _ as *mut c_void);
		}
	}

	#[allow(unused_variables)]
	pub fn stop(&mut self, gpio: &mut GPIO) {
		unsafe { gen_stop(self.raw); }
	}

	/*
	pub fn clear(&mut self) {
		unsafe { gen_clear(self.raw); }
	}

	pub fn position(&self) -> u32 {
		unsafe { gen_position(self.raw) as u32 }
	}
	*/
}

impl Drop for Generator {
	fn drop(&mut self) {
		unsafe { gen_free(self.raw); }
	}
}