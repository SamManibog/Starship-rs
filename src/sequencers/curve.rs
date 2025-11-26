use std::f64;

/// the shape of an easing function
#[derive(Debug, PartialEq)]
pub enum Smoothing {
    /// Sinusoid
    Sine,

    /// Cubic bezier
    Cubic,

    /// Quartic bezier
    Quartic,

    /// Overshoot before going to target
    Back(f64),
}

/// A possible easing function for a segment of a curve
#[derive(Debug, PartialEq)]
pub enum CurveShape {
    /// No easing function (jump to next once over)
    None,

    /// No smoothing
    Linear,

    /// Smoothing shape at start of transition
    In(Smoothing),

    /// Smoothing shape at end of transition
    Out(Smoothing),

    /// Smoothing shape at start and end of transition
    InOut(Smoothing)
}

impl CurveShape {
    pub fn interpolate(&self, x: f64, x_1: f64, x_2: f64, y_1: f64, y_2: f64) -> f64 {
		type S = Smoothing;
        match self {
            Self::None => {
                if x >= x_2 {
                    y_2
                } else {
                    y_1
                }
            }
            
            Self::Linear => {
                (x - x_1) * (y_2 - y_1) / (x_2 - x_1) + y_1
            }

            Self::In(S::Sine) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    1.0 - f64::cos((x * f64::consts::PI) / 2.0)
                })
            }

            Self::Out(S::Sine) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    f64::sin((x * f64::consts::PI) / 2.0)
                })
            }

            Self::InOut(S::Sine) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    (f64::cos(f64::consts::PI * x) - 1.0) / -2.0
                })
            }

            Self::In(S::Cubic) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    x.powi(3)
                })
            }

            Self::Out(S::Cubic) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    1.0 + (x - 1.0).powi(3)
                })
            }

            Self::InOut(S::Cubic) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    if x < 0.5 {
                        4.0 * x.powi(3)
                    } else {
                        1.0 - (-2.0 * x + 2.0).powi(3) / 2.0
                    }
                })
            }

            Self::In(S::Quartic) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    x.powi(4)
                })
            }

            Self::Out(S::Quartic) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    1.0 + (x - 1.0).powi(4)
                })
            }

            Self::InOut(S::Quartic) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    if x < 0.5 {
                        8.0 * x.powi(4)
                    } else {
                        1.0 - (-2.0 * x + 2.0).powi(4) / 2.0
                    }
                })
            }

            Self::In(S::Back(c)) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    (c + 1.0) * x * x * x - c * x * x
                })
            }

            Self::Out(S::Back(c)) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    1.0 + (c + 1.0) * (x - 1.0).powi(3) + c * (x - 1.0).powi(2)
                })
            }

            Self::InOut(S::Back(c)) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    let c2 = c * 1.525;
                    if x < 0.5 {
                        ( (2.0 * x).powi(2) * ( (c2 + 1.0) * 2.0 * x - c2 ) ) / 2.0
                    } else {
                        ( (2.0 * x - 2.0).powi(2) * ( (c2 + 1.0) * (x * 2.0 - 2.0) + c2 ) + 2.0) / 2.0
                    }
                })
            }

        }
    }

    /// takes a function with range and domain [0, 1]
    /// and uses it to interpolate between values
    fn generic_interpolate(
        &self,
        x: f64,
        x_1: f64,
        x_2: f64,
        y_1: f64,
        y_2: f64,
        func: impl Fn(f64) -> f64
    ) -> f64 {
        func((x - x_1) / (x_2 - x_1)) * (y_2 - y_1) + y_1
    }
}

/// A curve interpolating values of type T, stored with durations of type D
#[derive(Debug)]
pub struct Curve {
    /// there are n transitions
    /// the last transition may not be Easing::None
    /// we must guarantee that n >= 1
    transitions: Vec<CurveShape>,

    /// there are n + 1 values
    /// the values at i and i + 1
    /// correspond to the surrounding values of the transition
    values: Vec<f64>,

    /// there are n end times
    /// end_times[i] corresponds to transitions[i]
    /// Invariants:
    /// 	1) All values are positive
    /// 	2) end_times are sorted
    /// 	3) Values are unique
    end_times: Vec<f64>,
}

impl Curve {
    /// returns the value at the given time
    /// NOTE: if time is ZERO OR LESS, it will return the first value in the curve
    /// if time is greater than what the curve covers, it will return the last value in the curve
    /// O(log n)
    pub fn value_at_time(&self, time: f64) -> f64 {
        debug_assert!(time.is_finite() && time >= 0.0, "Time must be finite and non-negative.");

        // the index of the transition to use
        let index = match self.end_times.binary_search_by(|f| f.partial_cmp(&time).unwrap()) {
            Ok(i) | Err(i) => i,
        };

        // handle case where time is more than total duration
        if index >= self.end_times.len() {
            return *self.values.last().unwrap();
        }

        // lower, upper bounds
        let (y_1, y_2) = (self.values[index], self.values[index + 1]);

        // initial time
        let x_1 = if index >= 1 {
            self.end_times[index - 1]
        } else {
            0.0
        };

        // final time
        let x_2 = self.end_times[index];

        self.transitions[index].interpolate(time, x_1, x_2, y_1, y_2)
    }

    /// returns the total duration of the curve
    pub fn total_duration(&self) -> f64 {
        *self.end_times.last().unwrap()
    }
}

