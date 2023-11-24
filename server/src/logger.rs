//! Module for all Logger related things

use std::path::Path;

use colored::{
	Color,
	Colorize,
};
use flexi_logger::{
	style,
	DeferredNow,
	FileSpec,
	Logger,
	LoggerHandle,
	Record,
};

const LOG_TO_FILE: bool = false;
const LOG_FILE_COLOR: bool= false;
const LOG_FILE: &str = &"/tmp/termusic-server.log";

/// Function for setting up the logger
/// This function is mainly to keep the code structured and sorted
#[inline]
pub fn setup_logger() -> LoggerHandle {
	// TODO: look into https://github.com/emabee/flexi_logger/issues/142 again
	let handle = {
		let mut logger = Logger::try_with_env_or_str("warn")
			.expect("Expected flexi_logger to be able to parse env or string")
			.adaptive_format_for_stderr(flexi_logger::AdaptiveFormat::Custom(log_format, color_log_format))
			.log_to_stderr();

		if LOG_TO_FILE {
			if LOG_FILE_COLOR {
				logger = logger.format_for_files(color_log_format);
			} else {
				logger = logger.format_for_files(log_format);
			}

			let filespec = FileSpec::try_from(&LOG_FILE)
				.expect("Expected logging file to be parsed correctly");
			logger = logger
				.log_to_file(filespec)
				.append()
				.duplicate_to_stderr(flexi_logger::Duplicate::All);
		}

		logger.start().expect("Expected flexi_logger to be able to start")
	};

	// manually instead of "flexi_logger"'s "print_message", because that function is async and cannot be awaited, throwing off the rendered tui
	if LOG_TO_FILE {
		println!(
			"Logging to file \"{}\"",
			LOG_FILE
		);
	}

	handle.flush();

	return handle;
}

/// Logging format for log files and non-interactive formats
/// Not Colored and not padded
///
/// Example Lines:
/// `[2022-03-02T13:42:43.374+0100 ERROR module]: test line`
/// `[2022-03-02T13:42:43.374+0100 WARN module::deeper]: test line`
pub fn log_format(w: &mut dyn std::io::Write, now: &mut DeferredNow, record: &Record) -> Result<(), std::io::Error> {
	return write!(
		w,
		"[{} {} {}]: {}", // dont pad anything for non-interactive logs
		now.format_rfc3339(),
		record.level(),
		record.module_path().unwrap_or("<unnamed module>"),
		&record.args()
	);
}

/// Logging format for a tty for interactive formats
/// Colored and padded
///
/// Example Lines:
/// `[2022-03-02T13:42:43.374+0100 ERROR module]: test line`
/// `[2022-03-02T13:42:43.374+0100 WARN  module::deeper]: test line`
pub fn color_log_format(
	w: &mut dyn std::io::Write,
	now: &mut DeferredNow,
	record: &Record,
) -> Result<(), std::io::Error> {
	let level = record.level();
	return write!(
		w,
		"[{} {} {}]: {}",
		now.format_rfc3339().color(Color::BrightBlack), // Bright Black = Grey
		style(level).paint(format!("{level:5}")), // pad level to 2 characters, cannot be done in the string itself, because of the color characters
		record.module_path().unwrap_or("<unnamed module>"),
		&record.args() // dont apply any color to the input, so that the input can dynamically set the color
	);
}
