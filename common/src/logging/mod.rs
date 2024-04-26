#![deny(clippy::implicit_return)]
#![allow(clippy::needless_return)]

use std::ffi::OsStr;
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
	pub log_file_path: Result<OsString, Error>,
	pub time_is_local: bool
}

pub fn create_logger(module_path: Result<PathBuf, std::io::Error>, log_directory: Result<OsString, Error>) -> LoggersWithInfo
{
	let mut config = ConfigBuilder::new()
		.add_filter_allow("noblock_input_hook_injector".to_string())
		.add_filter_allow("noblock_input_hook".to_string())
		.add_filter_allow("noblock_input_common".to_string())
		.set_time_format_custom(format_description!("[[[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]]"))
		.set_thread_level(LevelFilter::Trace)
		.set_target_level(LevelFilter::Trace)
		.set_location_level(LevelFilter::Debug)
		.set_level_color(Level::Trace, Some(Color::Rgb(192, 192, 192)))
		.set_level_color(Level::Debug, Some(Color::Cyan))
		.set_level_color(Level::Info, Some(Color::Green))
		.set_level_color(Level::Warn, Some(Color::Yellow))
		.set_level_color(Level::Error, Some(Color::Red))
		.to_owned();
	let (config, time_is_local) = match config.set_time_offset_to_local()
	{
		Ok(local_time_config) => (local_time_config, true),
		Err(utc_time_config) => (utc_time_config, false)
	};
	let built_config = config.build();

	let tl = TermLogger::new(LevelFilter::Debug, built_config.to_owned(), TerminalMode::Mixed, ColorChoice::Auto);
	return match log_directory
	{
		Ok(log_directory) =>
		{
			let log_file_path = get_log_file_path(&module_path, log_directory);
			let log_file = OpenOptions::new().create(true).append(true).open(&log_file_path);
			match log_file
			{
				Ok(log_file) =>
				{
					let wl = WriteLogger::new(LevelFilter::Debug, built_config.to_owned(), log_file);
					LoggersWithInfo
					{
						loggers: vec![tl, wl],
						log_file_path: Ok(OsString::from(&log_file_path)),
						time_is_local
					}
				},
				Err(err) => LoggersWithInfo
				{
					loggers: vec![tl],
					log_file_path: Err(anyhow!("Could not open log file: {err}")),
					time_is_local
				}
			}
		},
		Err(err) =>
		{
			let err_message = err.to_string();
			LoggersWithInfo
			{
				loggers: vec![tl],
				log_file_path: Err(anyhow!(err_message)),
				time_is_local
			}
		}
	};
}

pub fn get_log_file_path(module_path: &Result<PathBuf, std::io::Error>, log_directory: OsString) -> PathBuf
{
	let pid = std::process::id();
	let formattable = format_description!("[year]-[month]-[day]_[hour]-[minute]-[second]");
	let now = OffsetDateTime::now_local().unwrap_or_else(|_| return OffsetDateTime::now_utc());
	let formatted_time = now.format(formattable).unwrap(); // the format description is hardcoded and known to be correct
	let module_name = match module_path
	{
		Ok(path) =>
		{
			let stem = match path.file_stem()
			{
				None => OsString::from("unknown"),
				Some(path_stem) => OsString::from(path_stem)
			};
			stem
		},
		Err(_) => OsString::from("unknown")
	};
	let pid_osstr = format!("{pid}");
	let pid_osstr = OsStr::new(&pid_osstr);
	let log_file_stem = [OsStr::new(&formatted_time), &module_name, &pid_osstr].join(OsStr::new("_"));
	let log_file_name = [log_file_stem.as_os_str(), OsStr::new("log")].join(OsStr::new("."));
	return Path::new(&log_directory).join(log_file_name);
}
