use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

mod decryptor;
mod analyzer;
mod reporter;

use decryptor::Decryptor;
use analyzer::Analyzer;
use reporter::Reporter;

fn main() {
    let args = Args::parse();
    
    println!("Exam Viewer Suite — Instructor Log Analyzer");
    println!("Author: A. Z. M. Arif | https://azmarif.dev");
    println!();
    
    if let Err(e) = run_command(args) {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}

#[derive(Parser)]
#[command(name = "exam-viewer")]
#[command(about = "Instructor-side decrypter, analyzer, and log viewer")]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Open and analyze an exam log file
    Open {
        /// Path to the encrypted ZIP file
        file: PathBuf,
    },
    /// Get summary only
    Summary {
        /// Path to the encrypted ZIP file
        file: PathBuf,
    },
    /// Verify integrity of exam log
    Verify {
        /// Path to the encrypted ZIP file
        file: PathBuf,
    },
    /// Export report to file
    Export {
        /// Path to the encrypted ZIP file
        file: PathBuf,
        /// Output format (pdf, markdown, json)
        #[arg(long)]
        pdf: Option<PathBuf>,
        #[arg(long)]
        markdown: Option<PathBuf>,
        #[arg(long)]
        json: Option<PathBuf>,
    },
}

fn run_command(args: Args) -> Result<()> {
    match args.command {
        Commands::Open { file } => {
            println!("Decrypting archive...");
            let decryptor = Decryptor::new(&file)?;
            let password = rpassword::prompt_password("Enter decryption password: ")?;
            
            println!("Verifying integrity...");
            let data = decryptor.decrypt(&password)?;
            
            println!("Generating session report...");
            let analyzer = Analyzer::new(data);
            let report = analyzer.analyze()?;
            
            let reporter = Reporter::new();
            reporter.print_full_report(&report)?;
            
            println!("\nDone.");
            Ok(())
        }
        Commands::Summary { file } => {
            let decryptor = Decryptor::new(&file)?;
            let password = rpassword::prompt_password("Enter decryption password: ")?;
            let data = decryptor.decrypt(&password)?;
            
            let analyzer = Analyzer::new(data);
            let report = analyzer.analyze()?;
            
            let reporter = Reporter::new();
            reporter.print_summary(&report)?;
            
            Ok(())
        }
        Commands::Verify { file } => {
            let decryptor = Decryptor::new(&file)?;
            let password = rpassword::prompt_password("Enter decryption password: ")?;
            
            match decryptor.verify_integrity(&password) {
                Ok(true) => {
                    println!("✓ Integrity check: PASSED");
                    Ok(())
                }
                Ok(false) => {
                    println!("✗ Integrity check: FAILED - File may have been tampered with!");
                    std::process::exit(1)
                }
                Err(e) => Err(e),
            }
        }
        Commands::Export { file, pdf, markdown, json } => {
            let decryptor = Decryptor::new(&file)?;
            let password = rpassword::prompt_password("Enter decryption password: ")?;
            let data = decryptor.decrypt(&password)?;
            
            let analyzer = Analyzer::new(data);
            let report = analyzer.analyze()?;
            
            let reporter = Reporter::new();
            
            if let Some(path) = pdf {
                reporter.export_pdf(&report, &path)?;
                println!("Report exported to: {}", path.display());
            }
            if let Some(path) = markdown {
                reporter.export_markdown(&report, &path)?;
                println!("Report exported to: {}", path.display());
            }
            if let Some(path) = json {
                reporter.export_json(&report, &path)?;
                println!("Report exported to: {}", path.display());
            }
            
            Ok(())
        }
    }
}

