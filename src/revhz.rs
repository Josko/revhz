extern crate getopts;
extern crate ioctl;

use std::env;
use std::fmt;
use std::f64;
use std::mem;
use std::str;
use std::io::prelude::*;
use std::fs::File;

use ioctl::libc::funcs::posix88::unistd::geteuid;

use getopts::Options;

const EVENTS: usize = 50;

struct Event {
  fd: i32,
  count: i32,
  avghz: i32,
  prvtime: f64,
  hz: Vec<i32>,
  name: Vec<u8>,
}

impl Default for Event {
    fn default() -> Event {
        Event { fd: 0, count: 0, avghz: 0, prvtime: 0.0, hz: Vec::with_capacity(64), name: Vec::with_capacity(128) }
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

  let mut events: [Event; EVENTS] = unsafe { mem::zeroed() };

  for event in 0 .. EVENTS {
    let device = fmt::format(format_args!("/dev/input/event{}", event));

    if let Ok(..) = File::open(&device) {
      unsafe {
        ioctl::eviocgname(events[event].fd, (events[event].name).as_mut_ptr(), 128);
      }

      println!("event: {}", events[event]);

      if verbose {
          println!("event{}: {}", event, str::from_utf8(&(events[event].name)).unwrap_or("ERROR"));
      }
    }
  }

  std::process::exit(0);
}
