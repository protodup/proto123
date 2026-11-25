//! FR-006 number display. All rounding is done on the decimal digit string,
//! half away from zero, so `0.1+0.2` shows `0.3` and never `0.30000000000000004`.

/// Significant digits of `|n|` rounded to 15 places: `digits` has no trailing zeros and
/// `|n| ≈ d.ddd × 10^exp`. `n` must be finite and nonzero.
fn sig15(n: f64) -> (Vec<u8>, i32) {
    let text = format!("{:.14e}", n.abs());
    let (mantissa, exp) = text.split_once('e').unwrap_or((text.as_str(), "0"));
    let exp: i32 = exp.parse().unwrap_or(0);
    let mut digits: Vec<u8> = mantissa.bytes().filter(u8::is_ascii_digit).map(|b| b - b'0').collect();
    while digits.len() > 1 && digits.last() == Some(&0) {
        digits.pop();
    }
    (digits, exp)
}

/// Keeps `keep` significant digits of `digits × 10^exp`, rounding half away from zero.
/// Returns an empty vector when the rounded value is zero.
fn round_sig(digits: &[u8], exp: i32, keep: i32) -> (Vec<u8>, i32) {
    if keep < 0 {
        return (Vec::new(), exp);
    }
    let keep = keep as usize;
    if keep == 0 {
        // Only the rounding digit remains: it rounds up to one unit at the new position.
        return match digits.first() {
            Some(&d) if d >= 5 => (vec![1], exp + 1),
            _ => (Vec::new(), exp),
        };
    }
    let mut kept: Vec<u8> = digits.iter().take(keep).copied().collect();
    kept.resize(keep, 0);
    let mut exp = exp;
    if digits.get(keep).copied().unwrap_or(0) >= 5 {
        let mut carry = true;
        for d in kept.iter_mut().rev() {
            if !carry {
                break;
            }
            if *d == 9 {
                *d = 0;
            } else {
                *d += 1;
                carry = false;
            }
        }
        if carry {
            kept.insert(0, 1);
            kept.pop();
            exp += 1;
        }
    }
    while kept.len() > 1 && kept.last() == Some(&0) {
        kept.pop();
    }
    if kept.iter().all(|&d| d == 0) {
        return (Vec::new(), exp);
    }
    (kept, exp)
}

/// Plain positional text (no sign) for `d.ddd × 10^exp`, trailing zeros removed.
fn fixed_text(digits: &[u8], exp: i32) -> String {
    let mut int_part = String::new();
    let mut frac_part = String::new();
    if exp >= 0 {
        let int_len = (exp + 1) as usize;
        for i in 0..int_len {
            let d = digits.get(i).copied().unwrap_or(0);
            int_part.push(char::from(b'0' + d));
        }
        for &d in digits.iter().skip(int_len) {
            frac_part.push(char::from(b'0' + d));
        }
    } else {
        int_part.push('0');
        for _ in 0..(-exp - 1) {
            frac_part.push('0');
        }
        for &d in digits {
            frac_part.push(char::from(b'0' + d));
        }
    }
    let trimmed = frac_part.trim_end_matches('0');
    if trimmed.is_empty() {
        int_part
    } else {
        format!("{int_part}.{trimmed}")
    }
}

/// Scientific text `d.ddd E±XX` (no sign), trailing zeros removed from the mantissa.
fn sci_text(digits: &[u8], exp: i32) -> String {
    let mut mantissa = String::new();
    mantissa.push(char::from(b'0' + digits.first().copied().unwrap_or(0)));
    let frac: String = digits.iter().skip(1).map(|&d| char::from(b'0' + d)).collect();
    let frac = frac.trim_end_matches('0');
    if !frac.is_empty() {
        mantissa.push('.');
        mantissa.push_str(frac);
    }
    let sign = if exp < 0 { '-' } else { '+' };
    format!("{mantissa}E{sign}{:02}", exp.unsigned_abs())
}

/// Full-precision form for the formula bar and live region: 15 significant digits,
/// trailing zeros removed, scientific only at very large or very small magnitudes.
pub fn format_full(n: f64) -> String {
    if !n.is_finite() {
        return "#VALUE!".to_owned();
    }
    if n == 0.0 {
        return "0".to_owned();
    }
    let sign = if n < 0.0 { "-" } else { "" };
    let (digits, exp) = sig15(n);
    let body = if (-9..15).contains(&exp) {
        fixed_text(&digits, exp)
    } else {
        sci_text(&digits, exp)
    };
    format!("{sign}{body}")
}

/// Grid display at `width` characters (FR-006): fixed with the most decimals that fit,
/// then scientific with the most mantissa digits that fit, then `#` fill.
pub fn format_general(n: f64, width: usize) -> String {
    if !n.is_finite() {
        return "#".repeat(width);
    }
    if n == 0.0 {
        return "0".to_owned();
    }
    let sign_len = usize::from(n < 0.0);
    let sign = if n < 0.0 { "-" } else { "" };
    let (digits, exp) = sig15(n);
    let natural_decimals = (digits.len() as i32 - 1 - exp).max(0);

    // Steps 1–2: fixed notation, most decimals first.
    for k in (0..=natural_decimals).rev() {
        let keep = exp + 1 + k;
        let (d, e) = round_sig(&digits, exp, keep);
        if d.is_empty() {
            break; // a nonzero value that rounds to zero: use scientific instead
        }
        let text = fixed_text(&d, e);
        if sign_len + text.len() <= width {
            return format!("{sign}{text}");
        }
    }

    // Step 3: scientific, most mantissa digits first.
    for m in (0..=14).rev() {
        let (d, e) = round_sig(&digits, exp, m + 1);
        if d.is_empty() {
            continue;
        }
        let text = sci_text(&d, e);
        if sign_len + text.len() <= width {
            return format!("{sign}{text}");
        }
    }

    // Step 4: nothing fits.
    "#".repeat(width)
}
