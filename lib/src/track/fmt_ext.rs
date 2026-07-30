use std::{fmt::Display, time::Duration};

/// Format the given Duration in the following way via a `Display` impl:
///
/// ```txt
/// # if Hours > 0
/// 10:01:01
/// # if Hour == 0
/// 01:01
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DurationFmtShort(pub Duration);

impl Display for DurationFmtShort {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let d = self.0;
        let duration_hour = d.as_secs() / 3600;
        let duration_min = (d.as_secs() % 3600) / 60;
        let duration_secs = d.as_secs() % 60;

        if duration_hour == 0 {
            write!(f, "{duration_min:0>2}:{duration_secs:0>2}")
        } else {
            write!(f, "{duration_hour}:{duration_min:0>2}:{duration_secs:0>2}")
        }
    }
}

impl DurationFmtShort {
    /// Get the value to display if no numbers are available.
    #[must_use]
    pub const fn fmt_empty() -> &'static str {
        "--:--"
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::super::DurationFmtShort;

    #[test]
    fn should_format_without_hours() {
        assert_eq!(
            DurationFmtShort(Duration::from_secs(61)).to_string(),
            "01:01"
        );
    }

    #[test]
    fn should_format_with_hours() {
        assert_eq!(
            DurationFmtShort(Duration::from_secs(60 * 61 + 1)).to_string(),
            "1:01:01"
        );
    }
}
