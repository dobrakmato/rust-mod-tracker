pub struct Fraction {
    numerator: usize,
    denominator: usize,
}

fn gcd(mut a: usize, mut b: usize) -> usize {
    let mut t = 0;
    while b != 0 {
        t = b;
        b = a % b;
        a = t;
    }
    return a;
}

impl Fraction {
    pub fn new(numerator: usize, denominator: usize) -> Self {
        let n = numerator;
        let d = denominator;

        let gcd = gcd(n, d);

        return Fraction {
            numerator: n / gcd,
            denominator: d / gcd,
        };
    }

    #[inline]
    fn denominator(&self) -> usize {
        return self.denominator;
    }

    #[inline]
    fn numerator(&self) -> usize {
        return self.numerator;
    }

    #[inline]
    fn as_float(&self) -> f64 {
        return self.numerator as f64 / self.denominator as f64;
    }
}

#[cfg(test)]
mod tests {
    use crate::math::Fraction;

    #[test]
    fn reduce() {
        let frac = Fraction::new(4, 2);
        assert_eq!(frac.numerator(), 2);
        assert_eq!(frac.denominator(), 1);

        let frac = Fraction::new(6, 9);
        assert_eq!(frac.numerator(), 2);
        assert_eq!(frac.denominator(), 3);

        let frac = Fraction::new(192000, 44100);
        assert_eq!(frac.numerator(), 640);
        assert_eq!(frac.denominator(), 147);
    }
}