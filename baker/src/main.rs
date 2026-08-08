use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("baker: {error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<(), String> {
    let mut args = env::args_os().skip(1);
    let command = args.next().ok_or_else(usage)?;
    let config = PathBuf::from(args.next().ok_or_else(usage)?);

    match command.to_str() {
        Some("check") => {
            if args.next().is_some() {
                return Err(usage());
            }
            let baked = baker::bake_path(&config).map_err(|error| error.to_string())?;
            println!(
                "{}: valid ({} LEDs, {} linecards, {} active patterns)",
                config.display(),
                baked.led_count(),
                baked.card_count(),
                baked.pattern_count()
            );
        }
        Some("render") => {
            let mut output = None;
            while let Some(arg) = args.next() {
                if arg != "--output" {
                    return Err(usage());
                }
                output = Some(PathBuf::from(args.next().ok_or_else(usage)?));
            }
            let source = baker::bake_to_string(&config).map_err(|error| error.to_string())?;
            if let Some(output) = output {
                fs::write(&output, source)
                    .map_err(|error| format!("write {}: {error}", output.display()))?;
            } else {
                print!("{source}");
            }
        }
        _ => return Err(usage()),
    }

    Ok(())
}

fn usage() -> String {
    "usage: baker check <config.toml> | render <config.toml> [--output <path>]".to_owned()
}
