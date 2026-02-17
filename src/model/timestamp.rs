use chrono::{DateTime, TimeZone, Utc};

pub fn detect_timestamp(line: &str) -> Option<DateTime<Utc>> {
    let patterns = [
        "%Y-%m-%dT%H:%M:%S%.f%:z",
        "%Y-%m-%dT%H:%M:%S%.3f%:z",
        "%Y-%m-%dT%H:%M:%S%.fZ",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
        "%d/%b/%Y:%H:%M:%S %z",
        "%Y/%m/%d %H:%M:%S",
        "%b %d %H:%M:%S",
    ];

    for pattern in patterns {
        if let Ok(dt) = DateTime::parse_from_str(line, pattern) {
            return Some(dt.with_timezone(&Utc));
        }
        if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(line, pattern) {
            return Some(Utc.from_utc_datetime(&naive));
        }
    }

    if line.starts_with('[') {
        if let Some(close_pos) = line.find(']') {
            let inner = &line[1..close_pos];
            for pattern in &patterns {
                if let Ok(dt) = DateTime::parse_from_str(inner, pattern) {
                    return Some(dt.with_timezone(&Utc));
                }
                if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(inner, pattern) {
                    return Some(Utc.from_utc_datetime(&naive));
                }
            }
        }
    }

    for (end_char, include_char) in [('Z', true), (' ', false)] {
        if let Some(pos) = line.find(end_char) {
            let end = if include_char { pos + 1 } else { pos };
            let prefix = &line[..end];
            for pattern in &patterns {
                if let Ok(dt) = DateTime::parse_from_str(prefix, pattern) {
                    return Some(dt.with_timezone(&Utc));
                }
                if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(prefix, pattern) {
                    return Some(Utc.from_utc_datetime(&naive));
                }
            }
        }
    }

    if let Some(pos) = line.find("+") {
        let prefix = &line[..pos + 6];
        for pattern in &patterns {
            if let Ok(dt) = DateTime::parse_from_str(prefix, pattern) {
                return Some(dt.with_timezone(&Utc));
            }
        }
    }
    if let Some(pos) = line.rfind("-") {
        if pos > 10 {
            let prefix = &line[..pos + 6];
            for pattern in &patterns {
                if let Ok(dt) = DateTime::parse_from_str(prefix, pattern) {
                    return Some(dt.with_timezone(&Utc));
                }
            }
        }
    }

    extract_iso_timestamp_prefix(line)
}

fn extract_iso_timestamp_prefix(line: &str) -> Option<DateTime<Utc>> {
    let patterns = [
        "%Y-%m-%dT%H:%M:%S%.fZ",
        "%Y-%m-%dT%H:%M:%SZ",
        "%Y-%m-%dT%H:%M:%S%.f",
        "%Y-%m-%dT%H:%M:%S",
        "%Y-%m-%d %H:%M:%S%.f",
        "%Y-%m-%d %H:%M:%S",
    ];

    for pattern in patterns {
        let fmt_len = estimate_format_len(pattern);
        if line.len() >= fmt_len {
            let prefix = &line[..fmt_len];
            if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(prefix, pattern) {
                return Some(Utc.from_utc_datetime(&naive));
            }
        }
    }

    None
}

fn estimate_format_len(fmt: &str) -> usize {
    let mut len = 0;
    let mut chars = fmt.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '%' {
            match chars.next() {
                Some('Y') => len += 4,
                Some('m' | 'd' | 'H' | 'M' | 'S') => len += 2,
                Some('.') => {
                    len += 1;
                    if chars.peek() == Some(&'f') {
                        chars.next();
                        len += 3;
                    }
                }
                Some('f') => len += 3,
                Some(_) => len += 1,
                None => break,
            }
        } else {
            len += c.len_utf8();
        }
    }
    len
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iso8601_with_timezone() {
        let line = "2026-02-13T10:30:45.123+00:00";
        let result = detect_timestamp(line);
        assert!(result.is_some());
    }

    #[test]
    fn test_iso8601_utc() {
        let line = "2026-02-13T10:30:45Z";
        let result = detect_timestamp(line);
        assert!(result.is_some());
    }

    #[test]
    fn test_common_log_format() {
        let line = "13/Feb/2026:10:30:45 +0000";
        let result = detect_timestamp(line);
        assert!(result.is_some());
    }

    #[test]
    fn test_no_timestamp() {
        let line = "This is just a log message without timestamp";
        let result = detect_timestamp(line);
        assert!(result.is_none());
    }

    #[test]
    fn test_datetime_space_separated() {
        let line = "2026-02-13 10:30:45";
        let result = detect_timestamp(line);
        assert!(result.is_some());
    }
}
