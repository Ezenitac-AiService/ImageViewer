//! Locale-neutral natural ordering (`image2 < image10`).

use std::cmp::Ordering;

pub struct NaturalSort;

impl NaturalSort {
    /// Compares two strings naturally: numeric digit runs compare numerically, text compares case-insensitively then exact.
    pub fn cmp(a: &str, b: &str) -> Ordering {
        let mut ia = a.chars().peekable();
        let mut ib = b.chars().peekable();

        loop {
            match (ia.peek().copied(), ib.peek().copied()) {
                (None, None) => return a.cmp(b),
                (None, Some(_)) => return Ordering::Less,
                (Some(_), None) => return Ordering::Greater,
                (Some(ca), Some(cb)) => {
                    if ca.is_ascii_digit() && cb.is_ascii_digit() {
                        let na = take_number(&mut ia);
                        let nb = take_number(&mut ib);
                        let ord = na.0.cmp(&nb.0).then_with(|| nb.1.cmp(&na.1));
                        if ord != Ordering::Equal {
                            return ord;
                        }
                    } else {
                        let la = ca.to_ascii_lowercase();
                        let lb = cb.to_ascii_lowercase();
                        let ord = la.cmp(&lb);
                        if ord != Ordering::Equal {
                            return ord;
                        }
                        ia.next();
                        ib.next();
                    }
                }
            }
        }
    }
}

fn take_number<I: Iterator<Item = char>>(iter: &mut std::iter::Peekable<I>) -> (u128, usize) {
    let mut val: u128 = 0;
    let mut len = 0usize;
    while let Some(&c) = iter.peek() {
        if let Some(d) = c.to_digit(10) {
            val = val.saturating_mul(10).saturating_add(u128::from(d));
            len += 1;
            iter.next();
        } else {
            break;
        }
    }
    (val, len)
}
