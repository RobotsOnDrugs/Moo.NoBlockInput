#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]


use std::ffi::OsString;
use std::fs::OpenOptions;
use std::path::Path;
use std::path::PathBuf;

use anyhow::anyhow;
use anyhow::Error;
use anyhow::Result;

use simplelog::Color;
use simplelog::ColorChoice;
use simplelog::ConfigBuilder;
use simplelog::format_description;
use simplelog::Level;
use simplelog::LevelFilter;
use simplelog::SharedLogger;
use simplelog::TerminalMode;
use simplelog::TermLogger;
use simplelog::WriteLogger;

use time::OffsetDateTime;

pub struct LoggersWithInfo
{
	pub loggers: Vec<Box<dyn SharedLogger>>,
	pub log_file_path: Result<OsString, Error>
}

pub fn create_logger(module_path: PathBuf, log_directory: Result<OsString, Error>) -> LoggersWithInfo
{
	let config = ConfigBuilder::new()
		.set_time_format_custom(format_description!("[[[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]]"))
		.set_thread_level(LevelFilter::Trace)
		.set_target_level(LevelFilter::Trace)
		.set_location_level(LevelFilter::Debug)
		.set_level_color(Level::Trace, Some(Color::Rgb(192,192,192)))
		.set_level_color(Level::Debug, Some(Color::Cyan))
		.set_level_color(Level::Info, Some(Color::Green))
		.set_level_color(Level::Warn, Some(Color::Yellow))
		.set_level_color(Level::Error, Some(Color::Red))
		.set_time_offset_to_local().unwrap()
		.build();
	let tl = TermLogger::new(LevelFilter::Debug, config.to_owned(), TerminalMode::Mixed, ColorChoice::Auto);
	return match log_directory
	{
		Err(err) => { LoggersWithInfo { loggers: vec![tl], log_file_path: Err(err) } }
		Ok(log_directory) =>
		{
			let log_file_name = get_log_file_name(module_path);
			let log_file_path = Path::new(&log_directory).join(log_file_name);
			let log_file = OpenOptions::new().create(true).append(true).open(&log_file_path);
			match log_file
			{
				Ok(log_file) =>
				{
					let wl = WriteLogger::new(LevelFilter::Debug, config, log_file);
					LoggersWithInfo { loggers: vec![tl, wl], log_file_path: Ok(OsString::from(&log_file_path)) }
				}
				Err(err) => { LoggersWithInfo { loggers: vec![tl], log_file_path: Err(anyhow!("Could not open log file: {err}")) } }
			}

		}
	};

}

pub fn get_log_file_name(module_path: PathBuf) -> String
{
	let pid = std::process::id();
	let formattable = format_description!("[year]-[month]-[day]_[hour]-[minute]-[second]");
	let now = OffsetDateTime::now_local().unwrap();
	let formatted_time = now.format(formattable).unwrap();
	let module_name = module_path.file_stem().unwrap().to_string_lossy();
	return format!("{formatted_time}_{module_name}_{pid}.log");
}