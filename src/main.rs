use std::env;
use std::process;

use placer::Config;

fn main() {
    let args: Vec<String> = env::args().collect();
    let config = Config::new(&args).unwrap_or_else(|err| {
        println!("Unable to parse arguments: {}", err);
        process::exit(1);
    });

    placer::run(&config).unwrap_or_else(|err| {
        println!("Failed to solve placements: {}", err);
        process::exit(1);
    });
}
