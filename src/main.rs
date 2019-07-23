use std::env;
use std::fs::File;
use std::io;
use std::io::Read;
use std::process;

pub use crate::config::*;
pub use crate::prompt::Prompt;

mod config;
pub mod parts;
mod prompt;

fn display_prompt(mut p: Prompt) {
    parts::exit_code(&mut p);
    parts::work_dir(&mut p);
    parts::git(&mut p);
    parts::dollar(&mut p);
    p.finish();
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.is_empty() {
        let config = match env::var("PROMPTRESS_CONFIG") {
            Ok(c) => c,
            Err(_) => {
                eprintln!("promptress: PROMPTRESS_CONFIG not set or contains invalid UTF-8");
                process::exit(1);
            }
        };
        let config: Config = match serde_json::from_str(&config) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("promptress: cannot parse PROMPTRESS_CONFIG: {}", e);
                process::exit(1);
            }
        };
        display_prompt(Prompt::new(config));
    } else if args.len() == 1 {
        let filename = &args[0];
        let mut file = match File::open(filename) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("promptress: cannot open {}: {}", filename, e);
                process::exit(1);
            }
        };
        let mut s = String::new();
        file.read_to_string(&mut s).unwrap();
        let config: Config = match toml::from_str(&s) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("promptress: cannot parse {}: {}", filename, e);
                process::exit(1);
            }
        };
        let stdout = io::stdout();
        serde_json::to_writer(stdout, &config).unwrap();
    } else {
        eprintln!("Usage: promptress [FILE]");
        eprintln!("Without FILE, render the prompt.");
        eprintln!("With FILE, compile the file configuration.");
        process::exit(2);
    }
}
