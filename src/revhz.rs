extern crate getopts;
extern crate ioctl;
extern crate nix;
extern crate num;

use num::{Num};
use nix::sys::signal;

use getopts::Options;

const EVENTS: usize = 50;

static mut quit: bool = false;

/// Zero-out a vector helper function.
fn zeros<T: Num>(size: usize) -> Vec<T> {
  let mut zero_vec: Vec<T> = Vec::with_capacity(size);

  for _ in 0 .. size {
    zero_vec.push(num::zero::<T>());
  }

  return zero_vec;
}

/// Event struct that will hold device data.
#[derive(Clone,Debug)]
struct Event {
  fd: i32,
  count: i32,
  avghz: i32,
  prvtime: f64,
  hz: Vec<i32>,
  name: Vec<u8>,
}

/// Default constructor.
impl Default for Event {
    fn default() -> Event {
        Event { fd: 0, count: 0, avghz: 0, prvtime: 0.0, hz: zeros::<i32>(64), name: zeros::<u8>(128) }
    }
}

/// Debug printing.
impl std::fmt::Display for Event {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "({}, {}, {}, {}, {:#?}, {:#?})", self.fd, self.count, self.avghz, self.prvtime, self.hz, self.name)
  }
}

/// Help text.
fn print_usage(program: &str, opts: Options) {
  let brief = format!("Usage: {} [options]", program);
  print!("{}", opts.usage(&brief));
}

/// Custom signar handler function.
extern fn handle_sigint(_: i32) {
  unsafe { quit = true };
}

fn main() {
  // Program arguments parsing.
  let args: Vec<String> = std::env::args().collect();
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

  // Superuser check.
  if unsafe { ioctl::libc::getuid() } != 0 {
    println!("{} must be used as superuser", program);
    std::process::exit(1);
  }

  // Set the signal handler so the user can exit.
  let sig_action = signal::SigAction::new(handle_sigint, signal::SockFlag::empty(), signal::SigSet::empty());
  match unsafe { signal::sigaction(signal::SIGINT, &sig_action) } {
    Ok(_) => {},
    Err(_) => {
      println!("Failure to set a signal handler");
      std::process::exit(1);
    }
  };

  // Get all input devices.
  let mut events: Vec<Event> = Vec::with_capacity(EVENTS);
  for event_number in 0 .. EVENTS {
    let mut event: Event = Event { ..Default::default() };

    let device = std::ffi::CString::new(std::fmt::format(format_args!("/dev/input/event{}", event_number))).unwrap();

    event.fd = unsafe { ioctl::libc::open(device.as_ptr(), ioctl::libc::O_RDONLY, 0) };

    if event.fd > 0 {
      unsafe { ioctl::eviocgname(event.fd, &mut (event.name[0]), 128) };

      if verbose {
          println!("event{}: {}", event_number, std::str::from_utf8(&(event.name)).unwrap_or("ERROR"));
      }

      events.push(event);
    }
  }

  println!("Press CTRL-C to exit.\n");

  // Block on events and read them until user prompts to quit.
  while unsafe{ !quit } {
    let mut set: ioctl::libc::fd_set = unsafe { std::mem::uninitialized() };
    unsafe {
      ioctl::libc::FD_ZERO(&mut set)
    };

    for event in &events {
      if event.fd != -1 {
        unsafe {
          ioctl::libc::FD_SET(event.fd, &mut set)
        };
      }
    }

    if unsafe { ioctl::libc::select(events.len() as i32,
                           &mut set,
                           std::ptr::null_mut::<ioctl::libc::fd_set>(),
                           std::ptr::null_mut::<ioctl::libc::fd_set>(),
                           std::ptr::null_mut::<ioctl::libc::timeval>()) } > 0 {
      for event_number in 0 .. EVENTS {
        if events[event_number].fd == -1 || unsafe { ioctl::libc::FD_ISSET(events[event_number].fd, &mut set) } {
          continue;
        }

        let mut input_event = ioctl::input_event { ..Default::default() };
        let bytes = unsafe { ioctl::libc::read(events[event_number].fd, &mut input_event as *mut _ as *mut ioctl::libc::c_void, std::mem::size_of::<ioctl::input_event>()) };
      }
    }
  }

  for event in &events {
    if event.fd != -1 {
      unsafe { ioctl::libc::close(event.fd) };
    }
  }

  std::process::exit(0);
}
