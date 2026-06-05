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
    ChebyshevFit, LogarithmicFit, assert_r_squared, basis_select, error::Error, plot, score::Aic, statistics::{CvStrategy, DegreeBound}
};

fn main() -> Result<(), Error> {
    // Let's load our noisy sample data again
    let data: Vec<(f64, f64)> = vec![
        (0.0, 1543.646446095297),
        (1.0, 2510.8448373886813),
        (2.0, -1049.8417777464808),
        (3.0, 128.8636916748329),
        (4.0, 2299.5789698162125),
        (5.0, -1088.877877984775),
        (6.0, 1618.2947969615125),
        (7.0, -2297.2422774360434),
        (8.0, -445.6256098008434),
        (9.0, -1604.41985213054),
        (10.0, -215.98750491947453),
        (11.0, 1839.7510630779911),
        (12.0, 791.2159509732394),
        (13.0, 2285.797983396175),
        (14.0, 2342.023738225539),
        (15.0, 3952.274413461916),
        (16.0, -288.71139955418676),
        (17.0, 3743.406065082094),
        (18.0, 1111.345903368434),
        (19.0, 1728.844832847898),
        (20.0, 2822.0269889370356),
        (21.0, 3145.1747058637347),
        (22.0, 2188.196423234208),
        (23.0, 786.5984170033012),
        (24.0, 3371.422775671008),
        (25.0, 1898.2173428824985),
        (26.0, 7010.027767042973),
        (27.0, 3645.5913846895783),
        (28.0, 4181.103887497001),
        (29.0, 4981.915341204722),
        (30.0, 3933.728797354763),
        (31.0, 6204.535093785302),
        (32.0, 3936.646543950818),
        (33.0, 5695.228056615476),
        (34.0, 6469.035843885751),
        (35.0, 5748.256598470418),
        (36.0, 5416.6856331907),
        (37.0, 7772.87511624726),
        (38.0, 9003.522539018504),
        (39.0, 9991.062118863527),
        (40.0, 10955.855448191593),
        (41.0, 11762.89109001035),
        (42.0, 7953.848180408664),
        (43.0, 11670.687716694321),
        (44.0, 10395.526316626017),
        (45.0, 11298.771814146092),
        (46.0, 11947.544580099056),
        (47.0, 9821.019769817289),
        (48.0, 14176.648308490909),
        (49.0, 12976.189065064911),
        (50.0, 13739.641745081097),
        (51.0, 14356.444779419984),
        (52.0, 13482.606267818432),
        (53.0, 13076.31642538684),
        (54.0, 14818.18208018875),
        (55.0, 15460.646618540484),
        (56.0, 19017.269577926265),
        (57.0, 14582.591618432272),
        (58.0, 18909.4252347379),
        (59.0, 19885.388160404895),
        (60.0, 16952.93183865643),
        (61.0, 22224.80937600329),
        (62.0, 19049.16311353027),
        (63.0, 20663.191139807),
        (64.0, 23302.781592122967),
        (65.0, 19523.708289856837),
        (66.0, 22560.451572723177),
        (67.0, 22666.010801678007),
        (68.0, 25756.910868975883),
        (69.0, 24113.234082118586),
        (70.0, 27252.40421500566),
        (71.0, 29018.04327205167),
        (72.0, 25675.165181528064),
        (73.0, 30274.294306420306),
        (74.0, 33112.08644525781),
        (75.0, 27022.05534247496),
        (76.0, 29529.926483603693),
        (77.0, 27313.961692659057),
        (78.0, 33122.85360192153),
        (79.0, 29642.288592215507),
        (80.0, 35709.91215472575),
        (81.0, 35566.19704411021),
        (82.0, 36452.24809112996),
        (83.0, 37289.18364553975),
        (84.0, 38928.358683976534),
        (85.0, 39541.47261817014),
        (86.0, 37240.91525661563),
        (87.0, 43026.43340830021),
        (88.0, 40489.53052146155),
        (89.0, 42255.37378499106),
        (90.0, 43005.386779409346),
        (91.0, 44309.23036636199),
        (92.0, 45635.46271624255),
        (93.0, 46533.44624833652),
        (94.0, 49017.4247539033),
        (95.0, 48547.48867469724),
        (96.0, 50075.02806958066),
        (97.0, 52383.262937175656),
        (98.0, 48299.53850795106),
        (99.0, 54095.627475042216),
    ];

    // Because this is the first time we've used this data, let's run `basis_select!` on it
    // Normally you'd do this from a #[test] function or the built in binary `basis_select` in this crate
    basis_select!(&data, DegreeBound::Relaxed, &Aic);

    // When you run this, you'll see (amonst other output) something like:
    // [ Evaluating 100 data points against 7 basis options ]
    //
    // # |             Basis              | Score Weight | Fit Quality | Normality | Rating
    // --|--------------------------------|--------------|-------------|-----------|-----------
    // 1 |                      Chebyshev |       20.00% |      67.92% |    47.88% | 63% ☆☆☆☆★
    // 2 |                       Legendre |       20.00% |      67.92% |    47.88% | 63% ☆☆☆☆★
    // 3 |          Probabilists' Hermite |       20.00% |      67.92% |    47.88% | 63% ☆☆☆☆★
    // --|--------------------------------|--------------|-------------|-----------|-----------
    // 4 |                       Laguerre |       20.00% |      67.92% |    47.88% | 63% ☆☆☆☆★
    // 5 |            Physicists' Hermite |       20.00% |      67.92% |    47.88% | 63% ☆☆☆☆★
    // 6 |                    Logarithmic |        0.00% |      67.51% |    65.50% | 67% ☆☆☆★★
    // 7 |                        Fourier |        0.00% |      86.77% |     0.00% | 65% ☆☆☆★★
    //
    // It's a confusing table but the important part is the ranking on the left.
    // Here we can see that Chebyshev, and a few others are tied for first place.
    // This makes sense because I used Chebyshev to generate the data!
    //
    //           Adjusted R² (How well the model fits the data)                 Likelihood that the errors are random, and not due to an unerlying pattern
    //   Likihood of being the best based on AIC               \               /            Combined ranking for fit quality, and normality of residuals
    //                                          \               |             |            /
    // # |             Basis              | Score Weight | Fit Quality | Normality | Rating
    // --|--------------------------------|--------------|-------------|-----------|-----------
    // 1 |                      Chebyshev |       20.00% |      67.92% |    47.88% | 63% ☆☆☆☆★
    //
    // But 2 other things stand out here:
    // - Although they produces worse scores with AIC, fourier has a better fit quality (R²) than Chebyshev1
    // - Logarithmic has a better normality score than Chebyshev, meaning the errors are more normally distributed
    //   That means the logarithmic fit is less likely to be overfitting the data, so it might generalize better to new data
    //
    // That being said, ideally you'd collect more data and run this a few times to see if the results are consistent.
    // Today we'll use Logarithmic because I want to show you how to use a different basis.
    //
    // I happen to know the data is a bit noisy, so I'll use k-fold cross validation to help avoid overfitting.
    // We will use it to minimize variance in the fit - this means we care more about getting a model that generalizes well to new data,
    // than we do about getting the best possible fit to this specific dataset.
    //
    // This will be 5-fold cross validation since we are using the `MinimizeVariance` strategy.
    //
    // 5-fold regression means I split the data into 5 parts, fit to 4/5 of it, and test on the remaining 1/5.
    // This is repeated 5 times, each time with a different 1/5 held out for testing.
    let fit = ChebyshevFit::new_auto(&data, DegreeBound::Relaxed, &Aic)?;

    // And of course don't forget to test!
    // Here we assert that the fit has an R² of at least 90%
    // If this fails, and the `plotting` feature is enabled, a plot will be generated to show you what went wrong.
    println!("\n\n");
    assert_r_squared!(fit);
    println!("Fitted Logarithmic Polynomial:\n  {fit}");
    plot!(fit);
    Ok(())
}
