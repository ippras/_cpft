//!
//! An example of using `basis_select!` to choose the best basis for fitting data.
//! This example loads some sample data from a JSON file, evaluates several basis options,
//! and fits a polynomial using the best basis according to AIC score.
//!
//! A basis is the set of functions used to build the polynomial. Different bases have different strengths and weaknesses.
//! - Stability: Some bases (like Chebyshev) are more numerically stable for large ranges of x-values.
//! - Fit Quality: Some bases (like Fourier) can fit certain types of data better.
//! - Outliers: Some bases are more robust to outliers in the data, like Logarithmic or Laguerre.
//! - Performance: Some bases are faster to compute than others, like Chebyshev or Legendre.
//!
use polyfit::{
    ChebyshevFit, LogarithmicFit, Polynomial, assert_r_squared,
    basis::MonomialBasis,
    basis_select,
    error::Error,
    plot,
    plotting::PlotOptions,
    score::{Aic, Bic},
    statistics::{CvStrategy, DegreeBound},
};

fn main() -> Result<(), Error> {
    // Let's load our noisy sample data again
    let data: Vec<(f64, f64)> = vec![
        (1.0, 18.397),
        (2.0, 18.442),
        (3.0, 18.473),
        (4.0, 18.49),
        (5.0, 18.508),
        (6.0, 18.519),
        (7.0, 18.529),
        (8.0, 18.538),
        (9.0, 18.545),
        (10.0, 18.561),
        (10.0, 18.553),
        (9.0, 18.541),
        (8.0, 18.534),
        (7.0, 18.525),
        (6.0, 18.518),
        (5.0, 18.503),
        (4.0, 18.487),
        (3.0, 18.467),
        (2.0, 18.44),
        (1.0, 18.392),
        (1.0, 18.394),
        (2.0, 18.439),
        (3.0, 18.467),
        (4.0, 18.49),
        (5.0, 18.505),
        (6.0, 18.516),
        (7.0, 18.529),
        (8.0, 18.537),
        (9.0, 18.548),
        (10.0, 18.553),
    ];

    // basis_select!(&data, DegreeBound::Relaxed, &Bic);

    println!("\n\n");
    let fit = LogarithmicFit::new_auto(&data, DegreeBound::Relaxed, &Aic)?;
    println!("Fitted Logarithmic Polynomial:\n  {fit}");
    println!("Coefficients:\n  {:?}", fit.coefficients());
    // plot!(fit);

    const T: u8 = 60;
    const NAME: &str = "18:1Δ9c";

    println!();
    let fit = LogarithmicFit::new_auto(&data, DegreeBound::Custom(1), &Aic)?;
    println!("Logarithmic:\n  {fit}");
    println!("Coefficients:\n  {:?}", fit.coefficients());
    plot!(fit, {
        title: format!("Logarithmic basis; {NAME}; {T}"),
        x_label: Some("X Axis".to_string()),
        y_label: Some("Y Axis".to_string()),
    });

    let fit = fit
        .into_polynomial()
        .project::<MonomialBasis<_>>(1.0..=3.0)?;
    println!("Polynomial:\n  {fit}");
    println!("Coefficients:\n  {:?}", fit.coefficients());
    plot!(fit, {
        title: format!("Monomial basis; {NAME}; {T}"),
        x_label: Some("X Axis".to_string()),
        y_label: Some("Y Axis".to_string()),
    }, prefix = "monomial");
    Ok(())
}
