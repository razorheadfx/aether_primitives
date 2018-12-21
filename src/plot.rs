
use crate::cf32;
use gnuplot::{AxesCommon, Caption, Color, Coordinate, DashType, Figure, LegendOption, LineStyle};

/// Plot a constellation diagram of the given symbols
pub fn constellation(symbols: &[cf32], title: &str, file: Option<&str>) {
    let mut fg = Figure::new();
    let re = symbols.iter().map(|c| c.re).collect::<Vec<_>>();
    let im = symbols.iter().map(|c| c.im).collect::<Vec<_>>();

    fg.axes2d()
        .points(&re, &im, &[Caption("Constellation"), Color("blue")])
        .set_legend(
            Coordinate::Graph(1.0),
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

#[cfg(feature = "fft")]
pub fn waterfall(symbols: &[cf32], fft_len: usize, use_db: bool, title: &str, file: Option<&str>) {
    use crate::fft::{Cfft, Scale};
    use crate::util::DB;
    use crate::vecops::VecOps;
    use gnuplot::AutoOption;

    let mut fg = Figure::new();

    let cols = fft_len;
    let rows = symbols.len() / fft_len;

    let mut fft = Cfft::with_len(fft_len);
    let scale = Scale::SN;

    let mut fft_ed = symbols.to_vec();

    // Not enough input symbols; Pad input to symbol.len() % fft_len == 0
    if fft_ed.len() % fft_len != 0 {
        let padding = fft_len - (fft_ed.len() % fft_len);
        (0usize..padding)
            .map(|_| cf32::default())
            .for_each(|c| fft_ed.push(c));
    }

    fft_ed.chunks_mut(fft_len).for_each(|c| {
        let _ = c.vec_rfft(&mut fft, scale).vec_mirror();
    });

    let levels: Vec<_> = if use_db {
        fft_ed
            .iter()
            .map(|c| c.norm())
            .map(|c| DB::from(c).db())
            .collect()
    } else {
        fft_ed.iter().map(|c| c.norm() as f64).collect()
    };

    fg.axes3d()
        .set_title(title, &[])
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

/// Plot of symbol real/imaginary parts with magnitude overview
pub fn time(symbol: &[cf32], title: &str, file: Option<&str>) {
    let mut fg = Figure::new();
    let x = (0..symbol.len()).collect::<Vec<_>>();
    let re = symbol.iter().map(|c| c.re).collect::<Vec<_>>();
    let im = symbol.iter().map(|c| c.im).collect::<Vec<_>>();
    let magn = symbol.iter().map(|c| c.norm()).collect::<Vec<_>>();

    fg.axes2d()
        .set_size(1.0, 0.75)
        .lines_points(&x, &re, &[Caption("real"), Color("blue")])
        .lines_points(&x, &im, &[Caption("imaginary"), Color("red")])
        .set_legend(
            Coordinate::Graph(1.0),
            Coordinate::Graph(1.0),
            &[LegendOption::Title(title)],
            &[],
        );
    fg.axes2d()
        .set_size(1.0, 0.25)
        .set_pos(0.0, 0.75)
        .lines(&x, &magn, &[Caption("magnitude"), Color("green")])
        .set_legend(
            Coordinate::Graph(1.0),
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

    let prep = |z: &[cf32]| {
        let re = z.iter().map(|c| c.re).collect::<Vec<_>>();
        let im = z.iter().map(|c| c.im).collect::<Vec<_>>();
        (re, im)
    };

    let err_magn = symbols1
        .iter()
        .zip(symbols2.iter())
        .map(|(a, b)| a - b)
        .map(|c| c.norm())
        .collect::<Vec<_>>();

    let (s1_re, s1_im) = prep(symbols1);
    let (s2_re, s2_im) = prep(symbols2);

    fg.axes2d()
        .set_size(1.0, 0.75)
        .lines(
            &x,
            &s1_re,
            &[
                Caption("Input 0: real"),
                Color("green"),
                LineStyle(DashType::Solid),
            ],
        )
        .lines(
            &x,
            &s1_im,
            &[
                Caption("Input 0: imaginary"),
                Color("green"),
                LineStyle(DashType::Dot),
            ],
        )
        .lines(
            &x,
            &s2_re,
            &[
                Caption("Input 1: real"),
                Color("blue"),
                LineStyle(DashType::Solid),
            ],
        )
        .lines(
            &x,
            &s2_im,
            &[
                Caption("Input 1: imaginary"),
                Color("blue"),
                LineStyle(DashType::Dot),
            ],
        )
        .set_legend(
            Coordinate::Graph(1.0),
            Coordinate::Graph(1.0),
            &[LegendOption::Title(title)],
            &[],
        );

    fg.axes2d()
        .set_size(1.0, 0.25)
        .set_pos(0.0, 0.75)
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
            Coordinate::Graph(1.0),
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

// TODO: add eye diagram

// TODO: add time/spectrum plot
