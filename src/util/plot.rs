extern crate gnuplot;

use crate::cf32;
use gnuplot::{
    AlignType, AutoOption, AxesCommon, Caption, Color, Coordinate, DashType, Figure, LegendOption,
    LineStyle,
};

/// Plot a constellation diagram of the given symbols
pub fn constellation(symbols: &[cf32], title: &str, file: Option<&str>) {
    let mut fg = Figure::new();
    let re = symbols.iter().map(|c| c.re);
    let im = symbols.iter().map(|c| c.im);

    fg.axes2d()
        .points(re, im, &[Caption("Constellation"), Color("blue")])
        .set_legend(
            Coordinate::Graph(0.5),
            Coordinate::Graph(1.0),
            &[
                LegendOption::Title(title),
                LegendOption::Placement(AlignType::AlignTop, AlignType::AlignLeft),
            ],
            &[],
        );
    match file {
        Some(filename) => {
            let _ = fg.set_terminal("pdfcairo", filename);
        }
        None => (),
    };
    fg.show();
}

#[cfg(feature = "fft")]
pub fn waterfall(symbols: &[cf32], fft_len: usize, use_db: bool, title: &str, file: Option<&str>) {
    use crate::fft::{Cfft, Scale};
    use crate::util::DB;
    use crate::vecops::VecOps;

    let mut fg = Figure::new();

    let cols = fft_len;
    let rows = symbols.len() / fft_len;

    let mut fft = Cfft::with_len(fft_len);
    let scale = Scale::SN;

    let fft_ed = {
        let mut padded = symbols.to_vec();
        // Not enough input symbols; Pad input to symbol.len() % fft_len == 0
        if padded.len() % fft_len != 0 {
            let padding = fft_len - (padded.len() % fft_len);
            (0usize..padding)
                .map(|_| cf32::default())
                .for_each(|c| padded.push(c));
        }
        // fft
        padded.chunks_mut(fft_len).for_each(|c| {
            let _ = c.vec_rfft(&mut fft, scale).vec_mirror();
        });
        padded
    };

    let levels = fft_ed.iter().map(|c| c.norm()).map(|c| match use_db {
        true => DB::from(c).db(),
        false => c as f64,
    });

    fg.axes3d()
        .set_title(title, &[])
        .set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(fft_len as f64))
        .surface(levels, rows, cols, None, &[])
        .set_view_map()
        .set_z_label(
            match use_db {
                true => "Magnitude [dB]",
                false => "Magnitude",
            },
            &[],
        )
        .set_z_range(AutoOption::Fix(0.0), AutoOption::Fix((fft_len - 1) as f64))
        .set_z_range(
            match use_db {
                true => AutoOption::Fix(0.0),
                false => AutoOption::Auto,
            },
            AutoOption::Auto,
        );

    match file {
        Some(filename) => {
            let _ = fg.set_terminal("pdfcairo", filename);
        }
        None => (),
    };

    fg.show();
}

#[cfg(feature = "fft")]
pub fn spectrum(symbols: &[cf32], fft_len: usize, use_db: bool, title: &str, file: Option<&str>) {
    use crate::fft::{Cfft, Scale};
    use crate::util::DB;
    use crate::vecops::VecOps;

    let mut fg = Figure::new();

    let mut fft = Cfft::with_len(fft_len);
    let scale = Scale::SN;

    let fft_ed = {
        let mut padded = symbols.to_vec();
        // Not enough input symbols; Pad input to symbol.len() % fft_len == 0
        if padded.len() % fft_len != 0 {
            let padding = fft_len - (padded.len() % fft_len);
            (0usize..padding)
                .map(|_| cf32::default())
                .for_each(|c| padded.push(c));
        }

        padded[0..fft_len].vec_rfft(&mut fft, scale);
        padded
    };

    let x = (0..fft_len).map(|x| x as f64);
    let norm = fft_ed.iter().map(|c| c.norm()).map(|v| match use_db {
        true => DB::from(v).db(),
        false => v as f64,
    });

    fg.axes2d()
        .set_size(1.0, 0.75)
        .set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(fft_len as f64))
        .lines_points(x, norm, &[Caption("Spectrum"), Color("green")])
        .set_legend(
            Coordinate::Graph(0.5),
            Coordinate::Graph(1.0),
            &[
                LegendOption::Title(title),
                LegendOption::Placement(AlignType::AlignTop, AlignType::AlignLeft),
            ],
            &[],
        );

    match file {
        Some(filename) => {
            let _ = fg.set_terminal("pdfcairo", filename);
        }
        None => (),
    };

    fg.show();
}

/// Plot of symbol real/imaginary parts with magnitude overview
pub fn time(symbol: &[cf32], title: &str, file: Option<&str>) {
    let mut fg = Figure::new();
    let x = (0..symbol.len()).collect::<Vec<_>>();
    let re = symbol.iter().map(|c| c.re);
    let im = symbol.iter().map(|c| c.im);
    let max = symbol
        .iter()
        .map(|c| c.norm())
        .fold(0f64, |a, b| match a > b as f64 {
            true => a,
            false => b as f64,
        })
        * 1.1;

    fg.axes2d()
        .set_size(1.0, 0.75)
        .lines_points(&x, re, &[Caption("Real"), Color("blue")])
        .lines_points(&x, im, &[Caption("Imaginary"), Color("red")])
        .set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(x.len() as f64))
        .set_y_range(AutoOption::Fix(-1.0 * max), AutoOption::Fix(max))
        .set_legend(
            Coordinate::Graph(0.5),
            Coordinate::Graph(1.00),
            &[
                LegendOption::Title(title),
                LegendOption::Horizontal,
                LegendOption::Placement(AlignType::AlignTop, AlignType::AlignLeft),
            ],
            &[],
        );

    // Calculate the max manually so even values exactly at the max are within axis range
    // instead of getting cut off
    let max = symbol
        .iter()
        .map(|c| c.norm())
        .fold(0f64, |a, b| match a > b as f64 {
            true => a,
            false => b as f64,
        })
        * 1.1;

    let magn = symbol.iter().map(|c| c.norm());
    fg.axes2d()
        .set_size(1.0, 0.25)
        .set_pos(0.0, 0.75)
        .lines(&x, magn, &[Caption("Magnitude"), Color("green")])
        .set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(x.len() as f64))
        .set_y_range(AutoOption::Fix(0.0), AutoOption::Fix(max))
        .set_legend(
            Coordinate::Graph(0.5),
            Coordinate::Graph(1.0),
            &[
                LegendOption::Title(title),
                LegendOption::Title(title),
                LegendOption::Placement(AlignType::AlignTop, AlignType::AlignLeft),
            ],
            &[],
        );

    match file {
        Some(filename) => {
            let _ = fg.set_terminal("pdfcairo", filename);
        }
        None => (),
    };

    fg.show();
}

/// Compare two symbol sequences
/// Plots real/imaginary and norm of error vector
pub fn compare(symbols1: &[cf32], symbols2: &[cf32], title: &str, file: Option<&str>) {
    assert_eq!(
        symbols1.len(),
        symbols2.len(),
        "Can only plot vectors of even length"
    );
    let mut fg = Figure::new();

    let x = (0..symbols1.len()).collect::<Vec<_>>();

    let err_magn = symbols1
        .iter()
        .zip(symbols2.iter())
        .map(|(a, b)| a - b)
        .map(|c| c.norm())
        .collect::<Vec<_>>();

    let s1_re = symbols1.iter().map(|c| c.re);
    let s1_im = symbols1.iter().map(|c| c.im);
    let s2_re = symbols2.iter().map(|c| c.re);
    let s2_im = symbols2.iter().map(|c| c.im);

    fg.axes2d()
        .set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(x.len() as f64))
        .set_size(1.0, 0.75)
        .lines(
            &x,
            s1_re,
            &[
                Caption("Input 0: real"),
                Color("green"),
                LineStyle(DashType::Solid),
            ],
        )
        .lines(
            &x,
            s1_im,
            &[
                Caption("Input 0: imaginary"),
                Color("green"),
                LineStyle(DashType::Dot),
            ],
        )
        .lines(
            &x,
            s2_re,
            &[
                Caption("Input 1: real"),
                Color("blue"),
                LineStyle(DashType::Solid),
            ],
        )
        .lines(
            &x,
            s2_im,
            &[
                Caption("Input 1: imaginary"),
                Color("blue"),
                LineStyle(DashType::Dot),
            ],
        )
        .set_legend(
            Coordinate::Graph(0.5),
            Coordinate::Graph(1.0),
            &[LegendOption::Title(title)],
            &[],
        );

    fg.axes2d()
        .set_size(1.0, 0.25)
        .set_pos(0.0, 0.75)
        .set_x_range(AutoOption::Fix(0.0), AutoOption::Fix(x.len() as f64))
        .lines(
            &x,
            &err_magn,
            &[
                Caption("Deviation"),
                Color("red"),
                LineStyle(DashType::DotDash),
            ],
        )
        .set_legend(
            Coordinate::Graph(0.5),
            Coordinate::Graph(1.0),
            &[LegendOption::Title(title)],
            &[],
        );

    match file {
        Some(filename) => {
            let _ = fg.set_terminal("pdfcairo", filename);
        }
        None => (),
    };

    fg.show();
}
