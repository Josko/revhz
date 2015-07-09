# revhz
Rust port of evhz program - https://github.com/ian-kelling/evhz

Made as a simple Rust language exercise by the repo owner and to test out the Cargo ecosystem.

To compile the binary simply execute at the root prooject directory:

    cargo build --release
  
Followed by executing the binary itself:

    sudo ./target/release/revhz
  
Note that the binary has to be run as a superuser to get refresh rates, in the same fashion as the original program.
