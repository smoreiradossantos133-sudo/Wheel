//! Advanced Math library for Wheel
//! Provides floating-point operations, trigonometry, and linear algebra

pub mod math {
    /// Sine function (radians)
    pub extern "C" fn math_sin(x: f64) -> f64 {
        x.sin()
    }

    /// Cosine function (radians)
    pub extern "C" fn math_cos(x: f64) -> f64 {
        x.cos()
    }

    /// Tangent function (radians)
    pub extern "C" fn math_tan(x: f64) -> f64 {
        x.tan()
    }

    /// Arc sine function
    pub extern "C" fn math_asin(x: f64) -> f64 {
        x.asin()
    }

    /// Arc cosine function
    pub extern "C" fn math_acos(x: f64) -> f64 {
        x.acos()
    }

    /// Arc tangent function
    pub extern "C" fn math_atan(x: f64) -> f64 {
        x.atan()
    }

    /// Two-argument arc tangent
    pub extern "C" fn math_atan2(y: f64, x: f64) -> f64 {
        y.atan2(x)
    }

    /// Square root
    pub extern "C" fn math_sqrt(x: f64) -> f64 {
        x.sqrt()
    }

    /// Power function (x^y)
    pub extern "C" fn math_pow(x: f64, y: f64) -> f64 {
        x.powf(y)
    }

    /// Exponential function (e^x)
    pub extern "C" fn math_exp(x: f64) -> f64 {
        x.exp()
    }

    /// Natural logarithm
    pub extern "C" fn math_log(x: f64) -> f64 {
        x.ln()
    }

    /// Base-10 logarithm
    pub extern "C" fn math_log10(x: f64) -> f64 {
        x.log10()
    }

    /// Absolute value
    pub extern "C" fn math_abs(x: f64) -> f64 {
        x.abs()
    }

    /// Ceiling function
    pub extern "C" fn math_ceil(x: f64) -> f64 {
        x.ceil()
    }

    /// Floor function
    pub extern "C" fn math_floor(x: f64) -> f64 {
        x.floor()
    }

    /// Round function
    pub extern "C" fn math_round(x: f64) -> f64 {
        x.round()
    }

    /// Modulo for floating-point
    pub extern "C" fn math_fmod(x: f64, y: f64) -> f64 {
        x % y
    }

    /// Minimum of two values
    pub extern "C" fn math_min(x: f64, y: f64) -> f64 {
        if x < y { x } else { y }
    }

    /// Maximum of two values
    pub extern "C" fn math_max(x: f64, y: f64) -> f64 {
        if x > y { x } else { y }
    }

    /// Convert integer to float (for LLVM IR)
    pub extern "C" fn math_int_to_float(i: i64) -> f64 {
        i as f64
    }

    /// Convert float to integer (truncate)
    pub extern "C" fn math_float_to_int(f: f64) -> i64 {
        f as i64
    }

    // Mathematical constants
    pub const PI: f64 = std::f64::consts::PI;
    pub const E: f64 = std::f64::consts::E;
    pub const TAU: f64 = std::f64::consts::TAU;
}
