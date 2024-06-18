use anyhow::{Context, Result};
use chrono::Local;
use clap::{Arg, Command};
use csv::Writer;
use log::{error, info};
use signal_hook::{consts::SIGINT, consts::SIGTERM, iterator::Signals};
use std::{
    fs::File,
    io::BufWriter,
    sync::atomic::{AtomicBool, Ordering},
    sync::Arc,
    thread,
    time::{Duration, Instant},
};
use sysinfo::{ProcessExt, System, SystemExt};

struct ProcessLogger {
    system: System,
    writer: Writer<BufWriter<File>>,
}

impl ProcessLogger {
    fn new(file_path: &str) -> Result<Self> {
        info!("Creating CSV file: {}", file_path);
        let file = File::create(file_path).context("Failed to create CSV file!")?;
        let writer = Writer::from_writer(BufWriter::new(file));
        info!("CSV file created successfully!");
        Ok(Self {
            system: System::new_all(),
            writer,
        })
    }

    fn write_header(&mut self) -> Result<()> {
        info!("Writing CSV header...");
        self.writer
            .write_record([
                "Timestamp",
                "PID",
                "Process Name",
                "CPU Usage (%)",
                "Memory Usage (%)",
            ])
            .context("Failed to write header")?;
        self.writer.flush().context("Failed to flush writer!")?;
        info!("CSV header written successfully!");
        Ok(())
    }

    fn log_processes(&mut self) -> Result<()> {
        self.system.refresh_all();
        let timestamp = Local::now().to_rfc3339();

        for (pid, process) in self.system.processes() {
            let name = process.name();
            let cpu_usage = process.cpu_usage();
            let memory_usage = process.memory() as f64 / self.system.total_memory() as f64 * 100.0;

            self.writer
                .write_record([
                    timestamp.as_str(),
                    &pid.to_string(),
                    name,
                    &format!("{:.2}", cpu_usage),
                    &format!("{:.2}", memory_usage),
                ])
                .context("Failed to write record!")?;
        }

        self.writer.flush().context("Failed to flush writer!")?;
        Ok(())
    }
}

struct Config {
    interval: u64,
    output: String,
    duration: u64,
}

impl Config {
    fn from_args(matches: &clap::ArgMatches) -> Result<Self> {
        let interval = *matches
            .get_one::<u64>("interval")
            .context("Invalid interval value")?;
        let output = matches.get_one::<String>("output").unwrap().clone();
        let duration = *matches
            .get_one::<u64>("duration")
            .context("Invalid duration value")?;

        Ok(Self {
            interval,
            output,
            duration,
        })
    }

    fn parse_args() -> clap::ArgMatches {
        Command::new("Process Logger")
            .version("1.0.1")
            .author("Jacob Coleman <jacob.wade.coleman@gmail.com>")
            .about("Writes process CPU and memory usage to a CSV file")
            .arg(
                Arg::new("interval")
                    .short('i')
                    .long("interval")
                    .value_name("SECONDS")
                    .help("Sets the logging interval in seconds")
                    .value_parser(clap::value_parser!(u64))
                    .default_value("1"),
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("FILE")
                    .help("Sets the output CSV file")
                    .default_value("process_usage.csv"),
            )
            .arg(
                Arg::new("duration")
                    .short('d')
                    .long("duration")
                    .value_name("SECONDS")
                    .help("Sets the maximum duration to run in seconds")
                    .value_parser(clap::value_parser!(u64))
                    .default_value("60"),
            )
            .get_matches()
    }
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let matches = Config::parse_args();
    let config = Config::from_args(&matches)?;
    info!(
        "Starting process logger with interval: {}s, output: {}, duration: {}s",
        config.interval, config.output, config.duration
    );

    let mut logger = ProcessLogger::new(&config.output)?;
    logger.write_header()?;

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    let mut signals = Signals::new([SIGINT, SIGTERM])?;
    thread::spawn(move || {
        for _ in signals.forever() {
            info!("Received termination signal, stopping...");
            r.store(false, Ordering::SeqCst);
        }
    });

    info!(
        "Writing process information every {} second(s) for {} second(s)...",
        config.interval, config.duration
    );

    let start_time = Instant::now();

    let result = run_logging_loop(
        &mut logger,
        &running,
        config.interval,
        config.duration,
        start_time,
    );

    match result {
        Ok(_) => info!("Process information gathered!"),
        Err(e) => error!("Process logging interrupted: {}", e),
    }

    Ok(())
}

fn run_logging_loop(
    logger: &mut ProcessLogger,
    running: &Arc<AtomicBool>,
    interval: u64,
    duration: u64,
    start_time: Instant,
) -> Result<()> {
    while running.load(Ordering::SeqCst) && start_time.elapsed() < Duration::from_secs(duration) {
        logger.log_processes()?;
        thread::sleep(Duration::from_secs(interval));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use tempfile::NamedTempFile;

    #[test]
    fn test_process_logger_creation() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let logger = ProcessLogger::new(file_path);
        assert!(logger.is_ok(), "Failed to create ProcessLogger");
    }

    #[test]
    fn test_write_header() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let mut logger = ProcessLogger::new(file_path).expect("Failed to create ProcessLogger");
        let result = logger.write_header();
        assert!(result.is_ok(), "Failed to write header");

        let file = File::open(file_path).expect("Failed to open temp file");
        let reader = BufReader::new(file);
        let header = reader
            .lines()
            .next()
            .expect("No header found")
            .expect("Failed to read header");
        assert_eq!(
            header,
            "Timestamp,PID,Process Name,CPU Usage (%),Memory Usage (%)"
        );
    }

    #[test]
    fn test_log_processes() {
        let temp_file = NamedTempFile::new().expect("Failed to create temp file");
        let file_path = temp_file.path().to_str().unwrap();

        let mut logger = ProcessLogger::new(file_path).expect("Failed to create ProcessLogger");
        logger.write_header().expect("Failed to write header");

        let result = logger.log_processes();
        assert!(result.is_ok(), "Failed to log processes");

        let file = File::open(file_path).expect("Failed to open temp file");
        let reader = BufReader::new(file);
        let lines: Vec<String> = reader
            .lines()
            .map(|line| line.expect("Failed to read line"))
            .collect();
        assert!(lines.len() > 1, "No process data logged");
    }

    #[test]
    fn test_config_from_args() {
        let args = vec![
            "process_logger",
            "--interval",
            "2",
            "--output",
            "test_output.csv",
            "--duration",
            "120",
        ];
        let matches = Command::new("Process Logger")
            .version("1.0.0")
            .author("Jacob Coleman <jacob.wade.coleman@gmail.com>")
            .about("Writes process CPU and memory usage to a CSV file")
            .arg(
                Arg::new("interval")
                    .short('i')
                    .long("interval")
                    .value_name("SECONDS")
                    .help("Sets the logging interval in seconds")
                    .value_parser(clap::value_parser!(u64))
                    .default_value("1"),
            )
            .arg(
                Arg::new("output")
                    .short('o')
                    .long("output")
                    .value_name("FILE")
                    .help("Sets the output CSV file")
                    .default_value("process_usage.csv"),
            )
            .arg(
                Arg::new("duration")
                    .short('d')
                    .long("duration")
                    .value_name("SECONDS")
                    .help("Sets the maximum duration to run in seconds")
                    .value_parser(clap::value_parser!(u64))
                    .default_value("60"),
            )
            .get_matches_from(args);

        let config = Config::from_args(&matches).expect("Failed to parse config from args");
        assert_eq!(config.interval, 2);
        assert_eq!(config.output, "test_output.csv");
        assert_eq!(config.duration, 120);
    }
}
