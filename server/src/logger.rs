//! Module for all Logger related things

use std::backtrace::Backtrace;

use colored::{Color, Colorize};
use flexi_logger::{DeferredNow, FileSpec, Logger, LoggerHandle, Record, style};

use crate::cli::Args;

/// Function for setting up the logger
/// This function is mainly to keep the code structured and sorted
#[inline]
pub fn setup(args: &Args) -> LoggerHandle {
    // TODO: look into https://github.com/emabee/flexi_logger/issues/142 again
    let handle = {
        let mut logger = Logger::try_with_env_or_str("warn")
            .expect("Expected flexi_logger to be able to parse env or string")
            .adaptive_format_for_stderr(flexi_logger::AdaptiveFormat::Custom(
                log_format,
                color_log_format,
            ))
            .panic_if_error_channel_is_broken(false)
            .log_to_stderr();

        if args.log_options.log_to_file {
            if args.log_options.file_color_log {
                logger = logger.format_for_files(color_log_format);
            } else {
                logger = logger.format_for_files(log_format);
            }

            let filespec = FileSpec::try_from(&args.log_options.log_file)
                .expect("Expected logging file to be parsed correctly");
            logger = logger
                .log_to_file(filespec)
                .append()
                .duplicate_to_stderr(flexi_logger::Duplicate::All);
        }

        logger
            .start()
            .expect("Expected flexi_logger to be able to start")
    };

    // manually instead of "flexi_logger"'s "print_message", because that function is async and cannot be awaited, throwing off the rendered tui
    if args.log_options.log_to_file {
        println!(
            "Logging to file \"{}\"",
            args.log_options.log_file.to_string_lossy()
        );
    }

    handle.flush();

    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |panic| {
        // this works because rust will execute the panic hook before unwinding
        let backtrace = Backtrace::capture();
        error!("Panic occured:\n{panic}\n{backtrace}");
        original_hook(panic);
    }));

    handle
}

/// Logging format for log files and non-interactive formats
/// Not Colored and not padded
///
/// Example Lines:
/// `[2022-03-02T13:42:43.374+0100 ERROR module]: test line`
/// `[2022-03-02T13:42:43.374+0100 WARN module::deeper]: test line`
pub fn log_format(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record<'_>,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "[{} {} {}]: {}", // dont pad anything for non-interactive logs
        now.format_rfc3339(),
        record.level(),
        record.module_path().unwrap_or("<unnamed module>"),
        &record.args()
    )
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
    record: &Record<'_>,
) -> Result<(), std::io::Error> {
    let level = record.level();
    write!(
        w,
        "[{} {} {}]: {}",
        now.format_rfc3339().color(Color::BrightBlack), // Bright Black = Grey
        style(level).paint(format!("{level:5}")), // pad level to 2 characters, cannot be done in the string itself, because of the color characters
        record.module_path().unwrap_or("<unnamed module>"),
        &record.args() // dont apply any color to the input, so that the input can dynamically set the color
    )
}
