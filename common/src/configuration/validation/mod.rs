use std::ffi::OsString;
use std::io::ErrorKind;
use std::path::Path;

use anyhow::anyhow;
use anyhow::Error;

const BAD_CHARACTERS: &[char] = &['<', '>', '"', '|', '?', '*'];

// This seems overly complicated, but I can't figure out how to make it easier to follow
// The result from the config
pub(crate) fn validate_log_directory(log_directory: anyhow::Result<OsString, Option<std::io::Error>>) -> anyhow::Result<OsString, Error>
{
	return match log_directory
	{
		Ok(log_directory) => validate_log_directory_config_value(Ok(log_directory)),
		Err(err) => match err
		{
			None => Err(anyhow!("Log directory entry in the configuration file was empty.")),
			Some(err) => validate_log_directory_config_value(Err(err))
		}
	};
}

fn validate_log_directory_config_value(result: anyhow::Result<OsString, std::io::Error>) -> anyhow::Result<OsString, Error>
{
	return match result
	{
		Ok(log_directory) =>
		{
			match log_directory.is_empty()
			{
				true => Err(anyhow!("Log directory registry value was empty.")),
				false =>
				{
					let path_test_str = log_directory.to_string_lossy();
					if path_test_str.rfind(BAD_CHARACTERS).is_some() | !matches!(path_test_str.rfind(':'), None | Some(1))
					{
						return Err(anyhow!("The specified log directory contains invalid characters."));
					}
					let path_test = Path::new(&log_directory);
					if path_test.is_relative()
					{
						return Err(anyhow!("The log directory path specified is relative. This leads to errors for hooked executables that can't write to their own parent directories."));
					}
					return match std::fs::metadata(&log_directory)
					{
						Ok(metadata) =>
						{
							if !metadata.is_dir() { return Err(anyhow!("The log directory path specified is not a directory.")); }
							Ok(log_directory)
						}
						Err(err) =>
						{
							if err.kind().ne(&ErrorKind::NotFound) { return Err(anyhow!("The log directory path is inaccessible: {err}")); }
							Ok(log_directory)
						}
					}
				}
			}
		}
		Err(err) =>
		{
			if err.kind().eq(&ErrorKind::NotFound) { return Err(anyhow!("The log directory option is not set.")); }
			return Err(anyhow!("There was an error trying to retrieve the log directory value: {err}"));
		}
	};
}