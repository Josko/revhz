extern crate getopts;
extern crate ioctl;

use std::env;
use ioctl::libc::funcs::posix88::unistd::geteuid;

use getopts::Options;

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

  let mut uid: u32;

  unsafe {
    uid = ioctl::libc::funcs::posix88::unistd::geteuid();
  }

  if uid != 0 {
    println!("{} must be used as superuser", program);
    std::process::exit(1);
  }

  std::process::exit(0);
}
