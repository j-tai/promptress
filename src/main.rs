use std::path::Path;
use std::env;
use std::env::VarError;
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
    parts::dollar(&mut p);
    p.finish();
}

/// Reads from a file. If this fails, prints the error and exits the
/// program.
fn read_file(filename: impl AsRef<Path>) -> String {
    let filename = filename.as_ref();
    let mut file = match File::open(filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("promptress: cannot open {}: {}", filename.display(), e);
            process::exit(1);
        }
    };
    let mut s = String::new();
    match file.read_to_string(&mut s) {
        Ok(_) => {}
        Err(e) => {
            eprintln!("promptress: cannot read from {}: {}", filename.display(), e);
            process::exit(1);
        }
    };
    s
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    if args.is_empty() {
        // Display the prompt
        match env::var("PROMPTRESS_CONFIG") {
            Ok(conf) => {
                let config: Config = match serde_json::from_str(&conf) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("promptress: cannot parse PROMPTRESS_CONFIG: {}", e);
                        process::exit(1);
                    }
                };
                display_prompt(Prompt::new(config));
            }
            Err(VarError::NotPresent) => {
                // If PROMPTRESS_CONFIG is not set, then read from .promptress.toml
                let mut path = dirs::home_dir().expect("cannot get home dir");
                path.push(".promptress.toml");
                let conf = read_file(&path);
                let config: Config = match toml::from_str(&conf) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("promptress: cannot parse {}: {}", path.display(), e);
                        process::exit(1);
                    }
                };
                display_prompt(Prompt::new(config));
            }
            Err(VarError::NotUnicode(_)) => {
                // Invalid Unicode -- just print the error and exit
                eprintln!("promptress: PROMPTRESS_CONFIG contains invalid unicode");
                process::exit(1);
            }
        };
    } else if args.len() == 1 {
        // Compile a config
        let s = read_file(&args[0]);
        let config: Config = match toml::from_str(&s) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("promptress: cannot parse {}: {}", args[0], e);
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
