use std::convert::{From, Into};

/// Decibel convert from DB and into DB
/// # Example
/// ```
/// use aether_primitives::util::DB;
/// let x = 100u32;
/// let db : DB = x.into();
/// assert_eq!(db.ratio(), 100f64);
/// // conversion works across types
/// assert_eq!(db.db(), 20f64);
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DB(f64);

impl<T> From<T> for DB
where
    T: Into<f64>,
{
    fn from(ratio: T) -> Self {
        DB(10f64 * ratio.into().log(10f64))
    }
}

impl DB {
    pub fn db(&self) -> f64 {
        self.0
    }

    pub fn ratio(&self) -> f64 {
        10f64.powf(self.0 / 10f64)
    }
}

#[cfg(test)]
mod test {
    use assert_approx_eq::assert_approx_eq;
    use crate::util::DB;

    #[test]
    fn conv_db_to_ratio() {
        let r: f64 = DB(30f64).ratio();
        assert_eq!(r, 1000f64);
        let r: f64 = DB(0f64).ratio();
        assert_eq!(r, 1f64);
    }

    #[test]
    fn conv_ratio_to_db() {
        assert_approx_eq!(DB::from(100f64).db(), 20f64);
        assert_approx_eq!(DB::from(10f64.recip()).db(), -10f64);
    }

}
