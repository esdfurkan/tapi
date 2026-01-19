#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use clap::Parser;
use std::path::Path;
use tapi_lib::{modes, utils};
mod server;
use tapi_lib::utils::logger::{ConsoleLogger, log_debug};

const AFTER_HELP: &str = "\
EXAMPLES:
  # Translate a folder using default settings (Grid Mode, Auto Columns)
   --folder /path/to/manga --api-key YOUR_API_KEY

  # Translate with specific model and languages
   --folder /path/to/manga --api-key KEY --model gemini-1.5-pro --target-lang tr

  # Disable Grid Mode (process images individually)
   --folder /path/to/manga --api-key KEY --no-grid

  # Archive Mode (process zip/cbz files)
   --folder /path/to/archives --api-key KEY --mode archive
";

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, after_help = AFTER_HELP)]
struct Args {
    /// Path to the folder to translate
    #[arg(short, long)]
    folder: Option<String>,

    /// API Key
    #[arg(short, long)]
    api_key: Option<String>,

    /// Model to use
    #[arg(short, long, default_value = "gemini-2.5-flash")]
    model: String,

    /// Target language
    #[arg(short, long, default_value = "en")]
    target_lang: String,

    /// Font to use
    #[arg(long, default_value = "wildwords")]
    font: String,

    /// Mode: cli or archive
    /// Mode: cli or archive
    #[arg(long, default_value = "cli")]
    mode: String,

    /// Run as Web Server
    #[arg(long)]
    server: bool,

    /// Server Port
    #[arg(long, default_value_t = 3000)]
    port: u16,

    /// Server Host
    #[arg(long, default_value = "0.0.0.0")]
    host: String,
}

fn main() {
    // Set up panic hook for debugging crashes
    std::panic::set_hook(Box::new(|info| {
        let msg = match info.payload().downcast_ref::<&str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &s[..],
                None => "Box<Any>",
            },
        };
        let location = info.location().map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column())).unwrap_or_else(|| "unknown".to_string());
        let log_msg = format!("CRITICAL PANIC at {}: {}", location, msg);
        eprintln!("{}", log_msg);
        log_debug(&log_msg);
    }));

    utils::logger::init_logger();
    
    // Check for CLI arguments
    let args = Args::parse();



    if args.server {
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            server::start_server(args.port, &args.host).await;
        });
        return;
    }

    if let Some(folder) = args.folder {
        if let Some(api_key) = args.api_key {
            // Run CLI mode
            let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
            rt.block_on(async {
                let logger = ConsoleLogger;
                let path = Path::new(&folder);
                
                println!("Running in CLI mode...");
                
                let result = if args.mode == "archive" {
                    modes::archive_mode::start_archive_translation(
                        &logger, 
                        path, 
                        &args.model, 
                        &api_key, 
                        &args.target_lang, 
                        &args.font,
                        "auto", // default text_align
                        false,
                        false,
                        12,
                        None,
                        None,
                        None
                    ).await
                } else {
                     modes::cli_mode::start_cli_translation(
                        &logger, 
                        path, 
                        &args.model, 
                        &api_key, 
                        &args.target_lang, 
                        &args.font,
                        "auto",
                        false,
                        false,
                        12,
                        None,
                        None,
                        None
                    ).await
                };

                match result {
                    Ok(_) => println!("Translation completed successfully."),
                    Err(e) => eprintln!("Error: {}", e),
                }
            });
        }
    } else {
        // Run UI mode
        tapi_lib::run();
    }
}

