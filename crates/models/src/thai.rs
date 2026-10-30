//! Thai and Buddhist-era formatting shared by the UI and the printed form.

use chrono::{Datelike, NaiveDate, NaiveTime, Timelike};

pub const THAI_MONTHS: [&str; 12] = [
    "มกราคม",
    "กุมภาพันธ์",
    "มีนาคม",
    "เมษายน",
    "พฤษภาคม",
    "มิถุนายน",
    "กรกฎาคม",
    "สิงหาคม",
    "กันยายน",
    "ตุลาคม",
    "พฤศจิกายน",
    "ธันวาคม",
];

pub fn be_year(ce_year: i32) -> i32 {
    ce_year + 543
}

pub fn thai_month_full(date: NaiveDate) -> &'static str {
    THAI_MONTHS
        .get(date.month0() as usize)
        .copied()
        .unwrap_or("")
}

/// "29 ตุลาคม 2569"
pub fn format_thai_date(date: NaiveDate) -> String {
    format!(
        "{} {} {}",
        date.day(),
        thai_month_full(date),
        be_year(date.year())
    )
}

/// "29 / 10 / 2569"
pub fn format_thai_date_slash(date: NaiveDate) -> String {
    format!(
        "{:02} / {:02} / {}",
        date.day(),
        date.month(),
        be_year(date.year())
    )
}

/// "16:00"
pub fn format_time_hm(time: NaiveTime) -> String {
    format!("{:02}:{:02}", time.hour(), time.minute())
}

/// Thai digits for the five preprinted row numbers.
pub fn thai_digit(value: usize) -> &'static str {
    match value {
        1 => "๑",
        2 => "๒",
        3 => "๓",
        4 => "๔",
        5 => "๕",
        6 => "๖",
        7 => "๗",
        8 => "๘",
        9 => "๙",
        _ => "๐",
    }
}

/// Formats a non-negative integer quantity.
pub fn format_qty_i64(qty: i64) -> String {
    qty.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).expect("valid test date")
    }

    #[test]
    fn buddhist_year_is_gregorian_plus_543() {
        assert_eq!(be_year(2026), 2569);
    }

    #[test]
    fn formats_thai_date() {
        assert_eq!(format_thai_date(date(2026, 10, 29)), "29 ตุลาคม 2569");
    }

    #[test]
    fn formats_thai_slash_date() {
        assert_eq!(format_thai_date_slash(date(2026, 1, 5)), "05 / 01 / 2569");
    }

    #[test]
    fn formats_time() {
        let time = NaiveTime::from_hms_opt(8, 5, 0).expect("valid time");
        assert_eq!(format_time_hm(time), "08:05");
    }

    #[test]
    fn thai_row_digits() {
        assert_eq!(thai_digit(1), "๑");
        assert_eq!(thai_digit(5), "๕");
    }

    #[test]
    fn formats_integer_quantity() {
        assert_eq!(format_qty_i64(30), "30");
        assert_eq!(format_qty_i64(0), "0");
    }
}
