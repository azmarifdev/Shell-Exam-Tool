use anyhow::Result;
use clap::Parser;
use std::process;

mod recorder;
mod encryption;
mod state;
mod metadata;

use recorder::Recorder;

fn main() {
    let _args = Args::parse();
    
    println!("Exam Recorder Suite — Student Terminal Session Recorder");
    println!("Author: A. Z. M. Arif  |  Website: https://azmarif.dev");
    println!();
    println!("Recording your exam session...");
    println!("All terminal activity is being securely logged.");
    println!("Type 'exit' to finish and generate your encrypted exam record.");
    println!();
    
    if let Err(e) = run_recorder() {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

#[derive(Parser)]
#[command(name = "exam-recorder")]
#[command(about = "Student-side secure terminal session recorder")]
struct Args {}

fn run_recorder() -> Result<()> {
    let mut recorder = Recorder::new()?;
    recorder.start()?;
    Ok(())
}

