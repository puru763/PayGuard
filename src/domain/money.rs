use serde::{Deserialize, Serialize};
use std::fmt;
use std::ops::{Add, Sub};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Paise(pub i64);

impl Paise {
    pub const ZERO: Paise = Paise(0);

    // Helper to create paise from rupees if needed
    pub fn from_rupees(rupees: i64) -> Self {
        Paise(rupees * 100)
    }

    pub fn to_rupees_f64(&self) -> f64 {
        self.0 as f64 / 100.0
    }
}

// Implement Addition safely with checked math to prevent silent overflows
impl Add for Paise {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        let res = self
            .0
            .checked_add(rhs.0)
            .expect("Integer overflow in money addition!");
        Paise(res)
    }
}

// Implement Subtraction safely
impl Sub for Paise {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        let res = self
            .0
            .checked_sub(rhs.0)
            .expect("Integer underflow in money subtraction!");
        Paise(res)
    }
}

// Pretty printing so it displays nicely as ₹
impl fmt::Display for Paise {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let rupees = self.0 / 100;
        let paisa = self.0 % 100;
        write!(f, "₹{}.{:02}", rupees, paisa.abs())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_money_math() {
        let m1 = Paise(50050);
        let m2 = Paise(10025);

        assert_eq!(m1 + m2, Paise(60075));
        assert_eq!(m1 - m2, Paise(40025));
    }

    #[test]
    fn test_display() {
        let m = Paise(164);
        assert_eq!(format!("{}", m), "₹1.64");
    }
}
