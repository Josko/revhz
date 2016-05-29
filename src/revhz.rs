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

    if unsafe { ioctl::libc::select(ioctl::libc::FD_SETSIZE as i32,
                                    &mut set,
                                    std::ptr::null_mut::<ioctl::libc::fd_set>(),
                                    std::ptr::null_mut::<ioctl::libc::fd_set>(),
                                    std::ptr::null_mut::<ioctl::libc::timeval>()) } > 0 {
      for event_number in 0 .. EVENTS - 1 {
        if events[event_number].fd == -1 || unsafe { ioctl::libc::FD_ISSET(events[event_number].fd, &mut set) } {
          continue;
        }

        let mut input_event = ioctl::input_event { ..Default::default() };
        let bytes = unsafe { ioctl::libc::read(events[event_number].fd, &mut input_event as *mut _ as *mut ioctl::libc::c_void, std::mem::size_of::<ioctl::input_event>()) };

        if bytes != std::mem::size_of::<ioctl::input_event>() as isize {
          continue;
        }

        // EV_REL = 0x02
        // EV_ABS = 0x03
        if input_event._type != 0x02 && input_event._type != 0x03 {
           let time: f64 = input_event.time.tv_sec as f64 * 1000.00 + (input_event.time.tv_usec / 1000) as f64;
           let hz: i64 = 1000 / (time - events[event_number].prvtime) as i64;

           if hz > 0 {
             let freq_index: usize = (events[event_number].count & (64 - 1)) as usize;
             events[event_number].count += 1;
             events[event_number].hz[freq_index] = hz as i32;
             events[event_number].avghz = 0;

             for freq in events[event_number].hz.clone() {
               events[event_number].avghz += freq;
             }

             if events[event_number].count > 64 {
               events[event_number].avghz /= 64;
             } else {
               events[event_number].avghz /= events[event_number].count;
             }

             if verbose {
               println!("{}: Latest {}Hz, Average {}Hz", std::str::from_utf8(&(events[event_number].name)).unwrap_or("ERROR"), hz, events[event_number].avghz);
             }
           }

           events[event_number].prvtime = time;
        }
      }
    }
  }

  // Cleanup.
  for event in &events {
    if event.fd != -1 {
      if event.avghz != 0 {
        println!("\nAverage for {}: {}Hz", std::str::from_utf8(&(event.name)).unwrap_or("ERROR"), event.avghz);
      }

      unsafe { ioctl::libc::close(event.fd) };
    }
  }

  std::process::exit(0);
}
