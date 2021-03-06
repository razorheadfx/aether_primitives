use std::convert::{From, Into};

/// Uses ```gnuplot``` to fork off threads to plot given data.  
/// If no filename is given to plot functions gnuplot will open
/// a window to display the plot.
#[cfg(feature = "plot")]
pub mod plot;

/// Operations on files of samples
pub mod file;

/// Convert values from Decibel (dB) and back
/// Stores the value in dB
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
pub struct DB(pub f64);

impl<T> From<T> for DB
where
    T: Into<f64>,
{
    #[inline]
    fn from(ratio: T) -> Self {
        DB(10f64 * ratio.into().log(10f64))
    }
}

impl DB {
    #[inline]
    pub fn db(self) -> f64 {
        self.0
    }

    #[inline]
    pub fn ratio(self) -> f64 {
        10f64.powf(self.0 / 10f64)
    }
}

#[cfg(test)]
mod test {
    use crate::util::DB;
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn db_to_ratio() {
        let r: f64 = DB(30f64).ratio();
        assert_eq!(r, 1000f64);
        let r: f64 = DB(0f64).ratio();
        assert_eq!(r, 1f64);
    }

    #[test]
    fn ratio_to_db() {
        assert_approx_eq!(DB::from(100f64).db(), 20f64);
        assert_approx_eq!(DB::from(10f64.recip()).db(), -10f64);
    }
}
