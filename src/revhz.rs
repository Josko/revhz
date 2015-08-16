extern crate getopts;
extern crate ioctl;
extern crate num;

use std::env;
use std::fmt;
use std::f64;
use std::mem;
use std::str;
use std::ffi::CString;
use std::io::prelude::*;
use std::fs::File;

use num::{Num, Zero, One};
use ioctl::libc::funcs::posix88::unistd::geteuid;

use getopts::Options;

const EVENTS: usize = 50;

#[derive(Clone,Debug)]
struct Event {
  fd: i32,
  count: i32,
  avghz: i32,
  prvtime: f64,
  hz: Vec<i32>,
  name: Vec<u8>,
}

fn zeros<T: Num>(size: usize) -> Vec<T> {
  let mut zero_vec: Vec<T> = Vec::with_capacity(size as usize);

  for i in 0..size {
    zero_vec.push(num::zero::<T>());
  }

  return zero_vec;
}

impl Default for Event {
    fn default() -> Event {
        Event { fd: 0, count: 0, avghz: 0, prvtime: 0.0, hz: zeros::<i32>(64), name: zeros::<u8>(128) }
    }
}

impl fmt::Display for Event {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "({}, {}, {}, {}, {:#?}, {:#?})", self.fd, self.count, self.avghz, self.prvtime, self.hz, self.name)
  }
}

fn print_usage(program: &str, opts: Options) {
  let brief = format!("Usage: {} [options]", program);
  print!("{}", opts.usage(&brief));
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let program = args[0].clone();

  let mut opts = Options::new();
  opts.optflag("n", "nonverbose", "do not print to stdout");
  opts.optflag("h", "help", "print this help menu");

  let matches = match opts.parse(&args[1..]) {
    Ok(m) => { m }
    Err(f) => { panic!(f.to_string()) }
  };

  if matches.opt_present("h") {
    print_usage(&program, opts);
    return;
  }

  let verbose: bool = !matches.opt_present("n");

  let mut uid: u32;

  unsafe {
    uid = ioctl::libc::funcs::posix88::unistd::geteuid();
  }

  if uid != 0 {
    println!("{} must be used as superuser", program);
    std::process::exit(1);
  }

  let mut events: Vec<Event> = Vec::with_capacity(EVENTS);

  for event_number in 0 .. EVENTS {
    let mut event: Event = Event { ..Default::default() };

    let device = CString::new(fmt::format(format_args!("/dev/input/event{}", event_number))).unwrap();

    event.fd = unsafe { ioctl::libc::open(device.as_ptr(), ioctl::libc::consts::os::posix88::O_RDONLY, 0) };

    if event.fd > 0 {
      let error_code = unsafe { ioctl::eviocgname(event.fd, &mut (event.name[0]), 128) };

      if verbose {
          println!("event{}: {}", event_number, str::from_utf8(&(event.name)).unwrap_or("ERROR"));
      }

      events.push(event);
    }
  }

  let mut quit = false;

  while !quit {
    break;
  }

  std::process::exit(0);
}
