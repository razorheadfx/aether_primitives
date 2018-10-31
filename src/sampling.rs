use crate::cf32;

/// linearly interpolate between samples in ```src``` and write the result to ```dst```
/// TODO: doc
pub fn interpolate(src: &[cf32], dst: &mut Vec<cf32>, n_between: usize) {
    src.windows(2).for_each(|s| {
        let x1 = s[0];
        let x2 = s[1];
        let rate = (
            (x2.re - x1.re) / (n_between + 1) as f32,
            (x2.im - x1.im) / (n_between + 1) as f32,
        );

        (0..=n_between).map(|i| i as f32).for_each(|i| {
            dst.push(cf32 {
                re: x1.re + i * rate.0,
                im: x1.re + i * rate.1,
            })
        });
    });
    dst.push(*src.last().unwrap());
}

/// downsample samples from ```src``` into ```dst``` with the ratio given by ```src.len()/dst.len()```
/// TODO: doc
pub fn downsample<T>(src: &[T], dst: &mut [T])
where
    T: Copy,
{
    debug_assert_eq!(
        src.len() % dst.len(),
        0,
        "Only even decimations are supported"
    );

    let dec = src.len() / dst.len();
    dst.iter_mut()
        .enumerate()
        .for_each(|(i, c)| *c = src[i * dec]);
}

/// uses the nightly step_by adaptor on iterator
/// TODO: doc
pub fn downsample_sb<T>(src: &[T], dst: &mut [T])
where
    T: Copy,
{
    debug_assert_eq!(
        src.len() % dst.len(),
        0,
        "Only even decimations are supported"
    );
    let dec = src.len() / dst.len();
    dst.iter_mut()
        .zip(src.iter().step_by(dec))
        .for_each(|(d, s)| *d = *s);
}


#[cfg(test)]
mod tests {

use crate::cf32;
use crate::sampling::interpolate;


    #[test]
    fn interpolate_2_between() {
        let src = [
            cf32::new(0f32, 0f32),
            cf32::new(3f32, 3f32),
            cf32::new(6f32, 6f32),
            cf32::new(9f32, 9f32),
        ];

        let mut dst = vec![];

        let interpolation = 2;
        interpolate(&src, &mut dst, interpolation);

        let check = [
            cf32::new(0f32, 0f32),
            cf32::new(1f32, 1f32),
            cf32::new(2f32, 2f32),
            cf32::new(3f32, 3f32),
            cf32::new(4f32, 4f32),
            cf32::new(5f32, 5f32),
            cf32::new(6f32, 6f32),
            cf32::new(7f32, 7f32),
            cf32::new(8f32, 8f32),
            cf32::new(9f32, 9f32),
        ];
        assert_eq!(dst.len(), src.len() + (src.len() - 1) * interpolation);

        assert_eq!(dst, check);
    }

    #[test]
    fn interpolate_1_between() {
        let src = [
            cf32::new(0f32, 0f32),
            cf32::new(2f32, 2f32),
            cf32::new(4f32, 4f32),
            cf32::new(6f32, 6f32),
        ];

        let mut dst = vec![];

        let interpolation = 1;
        interpolate(&src, &mut dst, interpolation);

        let check = [
            cf32::new(0f32, 0f32),
            cf32::new(1f32, 1f32),
            cf32::new(2f32, 2f32),
            cf32::new(3f32, 3f32),
            cf32::new(4f32, 4f32),
            cf32::new(5f32, 5f32),
            cf32::new(6f32, 6f32),
        ];
        assert_eq!(dst.len(), src.len() + (src.len() - 1) * interpolation);

        assert_eq!(dst, check);
    }
}