use std::{cmp::Ordering, f64};

/// a segment not associated with a curve
#[derive(Debug, Clone, PartialEq)]
pub struct OrphanedCurveSegment {
    pub start: (f64, f64),
    pub shape: CurveShape,
    pub end: (f64, f64),
}

/// the identifier for a segment in a curve unique within the curve that produced it
/// may become invalid after mutating the producing curve
/// may be invalid if used in a curve other than the producing curve
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CurveSegmentId {
    index: usize
}

/// the identifier for a point in a curve unique within the curve that produced it
/// may become invalid after mutating the producing curve
/// may be invalid if used in a curve other than the producing curve
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CurvePointId {
    index: usize,
    side: CurvePointSide
}

impl PartialOrd for CurvePointId {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for CurvePointId {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let index_cmp = self.index.cmp(&other.index);
        if index_cmp == Ordering::Equal {
            self.side.cmp(&other.side)
        } else {
            index_cmp
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CurvePointSide {
    /// the point is continuous (left and right are the same)
    Continuous,

    /// the left-hand limit of a discontinuity point
    Left,

    /// the right-hand limit of a discontinuity point
    Right,
}

impl PartialOrd for CurvePointSide {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(&other))
    }
}

impl Ord for CurvePointSide {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Self::Left => match other {
                Self::Left => Ordering::Equal,
                Self::Continuous | Self::Right => Ordering::Less,
            },
            
            Self::Continuous => match other {
                Self::Left => Ordering::Greater,
                Self::Continuous => Ordering::Equal,
                Self::Right => Ordering::Less,
            },

            Self::Right => match other {
                Self::Left | Self::Continuous => Ordering::Greater,
                Self::Right => Ordering::Equal,
            },
        }
    }
}

impl CurvePointSide {
    pub fn is_continuous(&self) -> bool {
        *self == CurvePointSide::Continuous
    }

    pub fn is_discontinuous(&self) -> bool {
        *self != CurvePointSide::Continuous
    }
}

/// the shape of an easing function
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SmoothingShape {
    /// Sinusoid
    Sine,

    /// Cubic bezier
    Cubic,

    /// Quartic bezier
    Quartic,

    // TODO: write function to get the range of this shape
    // /// Overshoot before going to target
    // Back(f32),
}

/// A possible easing function for a segment of a curve
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CurveShape {
    /// No smoothing
    Linear,

    /// use smoothing shape at start of transition
    In(SmoothingShape),

    /// use smoothing shape at end of transition
    Out(SmoothingShape),

    /// use smoothing shape at start and end of transition
    InOut(SmoothingShape)
}

impl CurveShape {
    pub fn interpolate(&self, x: f64, x_1: f64, x_2: f64, y_1: f64, y_2: f64) -> f64 {
		type S = SmoothingShape;
        match self {
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

            /*
            Self::In(S::Back(c)) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    let c = *c as f64;
                    (c + 1.0) * x * x * x - c * x * x
                })
            }

            Self::Out(S::Back(c)) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    let c = *c as f64;
                    1.0 + (c + 1.0) * (x - 1.0).powi(3) + c * (x - 1.0).powi(2)
                })
            }

            Self::InOut(S::Back(c)) => {
                self.generic_interpolate(x, x_1, x_2, y_1, y_2, |x| {
                    let c = *c as f64;
                    let c2 = c * 1.525;
                    if x < 0.5 {
                        ( (2.0 * x).powi(2) * ( (c2 + 1.0) * 2.0 * x - c2 ) ) / 2.0
                    } else {
                        ( (2.0 * x - 2.0).powi(2) * ( (c2 + 1.0) * (x * 2.0 - 2.0) + c2 ) + 2.0) / 2.0
                    }
                })
            }
*/

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

/// a struct representing a value at a point in time in a curve,
/// capable of handling a discontinuity
#[derive(Debug, Clone)]
pub struct CurveYValue {
    pub left_limit: f64,
    pub right_limit: f64
}

impl CurveYValue {
    pub fn new_single(value: f64) -> Self {
        Self {
            left_limit: value,
            right_limit: value
        }
    }

    pub fn new_double(left_limit: f64, right_limit: f64) -> Self {
        Self {
            left_limit,
            right_limit
        }
    }

    pub fn is_continuous(&self) -> bool {
        self.left_limit == self.right_limit
    }

    pub fn is_discontinuous(&self) -> bool {
        self.left_limit != self.right_limit
    }
}

/// A curve interpolating values of type T, stored with durations of type D
#[derive(Debug)]
pub struct Curve {
    /// there are n transitions such that n >= 1
    transitions: Vec<CurveShape>,

    /// there are n + 1 values for value[i],
    /// 	1) It occurs at end_times[i - 1] if i > 1 or at 0 if i = 1
    ///		2) It is transitioned into with shape transitions[i - 1]
    ///		3) It is transitioned out from with shape transitions[i]
    ///	Invariants:
    ///		1) The start and end yvalues are singles
    values: Vec<CurveYValue>,

    /// there are n end times
    /// end_times[i] corresponds to transitions[i] and values[i + 1]
    /// Invariants:
    /// 	1) All values are positive
    /// 	2) end_times are sorted
    /// 	3) Values are unique
    end_times: Vec<f64>,
}

impl Curve {
    /// creates a new curve with the given value and duration
    pub fn new(value: f64, duration: f64) -> Self {
        Self {
            transitions: vec![CurveShape::Linear],
            values: vec![CurveYValue::new_single(value), CurveYValue::new_single(value)],
            end_times: vec![duration]
        }
    }

    /// returns the value at the given time
    /// NOTE: if time is ZERO OR LESS, it will return the first value in the curve
    /// if time is greater than what the curve covers, it will return the last value in the curve
    /// O(log n)
    pub fn value_at_time(&self, time: f64) -> f64 {
        if time <= 0.0 {
            return self.values[0].right_limit;
        }

        // the index of the transition to use
        let index = match self.end_times.binary_search_by(|f| f.partial_cmp(&time).unwrap()) {
            Ok(i) => { return self.values[i + 1].right_limit; }
            Err(i) => i,
        };

        // handle case where time is more than total duration
        if index >= self.end_times.len() {
            return self.values.last().unwrap().right_limit;
        }

        let transition = self.transitions[index];

        // lower bound
        let y_1 = self.values[index].right_limit;

        // upper bound
        let y_2 = self.values[index + 1].left_limit;

        // initial time
        // note: we need bounds checking like this to avoid underflow
        let x_1 = if index > 0 {
            self.end_times[index - 1]
        } else {
            0.0
        };

        // final time
        let x_2 = self.end_times[index];

        transition.interpolate(time, x_1, x_2, y_1, y_2)
    }

    /// returns the total duration of the curve
    pub fn total_duration(&self) -> f64 {
        *self.end_times.last().unwrap()
    }

    /// returns the segment at the given time
    /// if the time lies on a point, defaults to the right side of the point
    /// if the time is less than 0, defaults to the first segment
    /// if the time is greater than or equal to the total duration, defaults to the last segment
    pub fn get_segment(&self, time: f64) -> CurveSegmentId {
        if time <= 0.0 {
            CurveSegmentId {
                index: 0,
            }
        } else if time >= self.total_duration() {
            CurveSegmentId {
                index: self.transitions.len() - 1,
            }
        } else {
            CurveSegmentId {
                index: match self.end_times.binary_search_by(|f| f.partial_cmp(&time).unwrap()) {
                    Ok(i) => i,
                    Err(i) => i - 1,
                },
            }
        }
    }

    /// returns the point nearest to the given time
    /// defaults to the right-hand limit
    pub fn get_nearest_point(&self, time: f64) -> CurvePointId {
        if time <= 0.0 {
            return CurvePointId {
                index: 0,
                side: CurvePointSide::Continuous,
            };
        }

        if time >= self.total_duration() {
            return CurvePointId {
                index: self.values.len() - 1,
                side: CurvePointSide::Continuous
            };
        }

        match self.end_times.binary_search_by(|f| f.partial_cmp(&time).unwrap()) {
            Ok(i) => CurvePointId {
                index: i + 1,
                side: if self.values[i + 1].is_continuous() {
                    CurvePointSide::Continuous
                } else {
                    CurvePointSide::Right
                },
            },

            Err(i) => if time - self.end_times[i] - time < self.end_times[i + 1] - time {
                CurvePointId {
                    index: i,
                    side: if self.values[i].is_continuous() {
                        CurvePointSide::Continuous
                    } else {
                        CurvePointSide::Right
                    }
                }

            } else {
                CurvePointId {
                    index: i + 1,
                    side: if self.values[i + 1].is_continuous() {
                        CurvePointSide::Continuous
                    } else {
                        CurvePointSide::Right
                    },
                }
            },
        }

    }

    /// deletes all segments between the given points (inclusive) and the following transition
    ///
    /// fails if this operation would leave less than two points in the curve
    ///
    /// if we delete the first point, all end times will be updated to start at 0 again
    ///
    /// returns None if the removal fails
    /// returns Some(i) where i is the difference in start time of the curve.
    /// note that i will be 0 if the first point was not deleted
    pub fn remove_point_to_point(&mut self, point1: CurvePointId, point2: CurvePointId) -> Option<f64> {
        debug_assert!(self.point_is_valid(point1), "point1 is not contained in the curve");
        debug_assert!(self.point_is_valid(point2), "point2 is not contained in the curve");

    	todo!("handle different cases for continuous and discontinuous points");

        // put points in order
        let (start, end) = if point1.index < point2.index {
            (point1, point2)
        } else {
            (point2, point1)
        };

        // ensure that we have selected a valid range
        if end.index - start.index + 1 >= self.values.len() - 2 {
            return None;
        }

        // handle case where we are removing a single point
        if start.index == end.index {
            return self.remove_point(start);
        }

        if start.index <= 0 {
            // handle case where we delete the start point
            let offset = self.end_times[end.index];

            // delete entries
            self.values.drain(0..=end.index);
            self.transitions.drain(0..=end.index);
            self.end_times.drain(0..=end.index);

            // make things start at 0 again
            self.end_times.iter_mut().for_each(|f| *f -= offset);

            // preserve the invariant that the start yvalue must be a single
            let start_y_val = self.values.first_mut().unwrap();
            start_y_val.left_limit = start_y_val.right_limit;

            return Some(offset);
        }

        if end.index >= self.values.len() - 1 {
            // handle case where we delete the end point
            self.values.drain(start.index..=end.index);
            self.transitions.drain(start.index - 1..=end.index - 1);
            self.end_times.drain(start.index - 1..=end.index - 1);

        } else {
            // handle case where we only delete intermediate points
            self.values.drain(start.index..=end.index);
            self.transitions.drain(start.index..=end.index);
            self.end_times.drain(start.index - 1..=end.index - 1);
        }

        // preserve invariant that the last curve segment must have the
        // same start and end values if it is CurveShape::None
        // note that we must handle this scenario for intermediate points as well,
        // as we may delete the last transition when deleting the second-to-last point
        let last = self.values.last_mut().unwrap();
        last.left_limit = last.right_limit;

        Some(0.0)
    }

    /// deletes the given point on the curve and the following transition
    ///
    /// fails if and only there are two points or less points on the curve
    ///
    /// if we delete the first point, all end times will be updated to start at 0 again
    ///
    /// returns None if the removal fails
    /// returns Some(i) where i is the change in start time of the curve
    /// note that i will be 0 if the first point was not deleted
    pub fn remove_point(&mut self, point: CurvePointId) -> Option<f64> {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");

        if self.values.len() <= 2 {
            return None;
        }

        if point.side.is_discontinuous() {
            //if we are on a discontinuous point, just delete the specified side
            let y_val = &mut self.values[point.index];
            if point.side == CurvePointSide::Right {
                y_val.right_limit = y_val.left_limit;
            } else {
                y_val.left_limit = y_val.right_limit;
            }
            return None;
        }
        
        if point.index <= 0 {
            let offset = self.end_times[0];

            // delete entries
            self.values.remove(0);
            self.transitions.remove(0);
            self.end_times.remove(0);

            // make things start at 0 again
            self.end_times.iter_mut().for_each(|f| *f -= offset);
            return Some(offset);
        }

        if point.index >= self.values.len() - 1 {
            // if we are dealing with the last point, just delete from the end

            self.transitions.pop();
            self.values.pop();
            self.end_times.pop();

        } else {
            // if we are dealing with an intermediate point

            // remove right transition
            self.transitions.remove(point.index);
            self.end_times.remove(point.index - 1);
            self.values.remove(point.index);

        }

        // preserve the invariant that the end yvalue must be a single
        let last = self.values.last_mut().unwrap();
        last.left_limit = last.right_limit;

        Some(0.0)
    }

    /// adds a point at the given time
    ///
    /// if we add in the middle of a transition, the point will be placed
    /// such that it is on the existing curve. it will also take on the 
    /// transitions it broke up on either side of it
    ///
    /// if we add after the total duration, we will add a linear transition from the end
    /// with the same output value as before
    ///
    /// fails if a point already exists at the given time
    ///
    /// returns the point added
    pub fn add_point(&mut self, time: f64) -> Option<CurvePointId> {
        if time == 0.0 {
            None

        } else if time < 0.0 {
            self.values.insert(0, CurveYValue::new_single(self.values.first().unwrap().left_limit));
            self.transitions.insert(0, CurveShape::Linear);
            self.end_times.insert(0, 0.0);

            self.end_times.iter_mut().for_each(|f| *f -= time);

            Some(CurvePointId { index: 0, side: CurvePointSide::Continuous })

        } else if let Err(index) = self.end_times.binary_search_by(|f| f.partial_cmp(&time).unwrap()) {
            // index is the index of the transition to split
            if index >= self.end_times.len() {
                // if we are after total duration, just append data
                self.values.push(self.values.last().unwrap().clone());
                self.transitions.push(CurveShape::Linear);
                self.end_times.push(time);

            } else {
                // if we are splitting a transition, perform insertions
                self.values.insert(index + 1, CurveYValue::new_single(self.value_at_time(time)));
                self.transitions.insert(index, self.transitions[index]);
                self.end_times.insert(index, time);
            }

            Some(CurvePointId { index: index + 1, side: CurvePointSide::Continuous })

        } else {
            None
        }
    }

    /// sets the value of the given point
    pub fn set_point_value(&mut self, point: CurvePointId, value: f64) {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");

        match point.side {
            CurvePointSide::Right => { self.values[point.index].right_limit = value; }
            CurvePointSide::Left => { self.values[point.index].left_limit = value; }
            CurvePointSide::Continuous => {
                self.values[point.index].right_limit = value;
                self.values[point.index].left_limit = value;
            }
        }
    }

    /// moves the time of the given point
    /// the point's time will not move outside of the surrounding points's times
    ///
    /// if the starting point was moved, all times will be adjusted so that
    /// the starting time is at zero
    ///
    /// returns the new id of the point
    pub fn set_point_time(&mut self, point: CurvePointId, time: f64) -> CurvePointId {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");

        if self.point_is_start(point) {
            // handle fusion
            if time == self.end_times[0] {
                self.end_times.remove(0);
                self.transitions.remove(0);
                self.values[1] = CurveYValue::new_single(self.get_point_value(point));
                self.values.remove(0);
            }

            self.end_times.iter_mut().for_each(|f| *f -= time);

            self.first_point()

        } else if self.point_is_end(point) {
            let min_time = self.end_times[self.end_times.len() - 2];
            if time <= min_time {
                let value = self.get_point_value(point);
                self.end_times.pop();
                self.transitions.pop();
                self.values.pop();
                *self.values.last_mut().unwrap() = CurveYValue::new_single(value);

            } else {
                *self.end_times.last_mut().unwrap() = time.max(min_time);
            }

            self.last_point()

        } else if point.side == CurvePointSide::Continuous {
            let min_time = if point.index == 1 {
                // if end_time of this point is at index 0,
                0.0
            } else {
                // if the end_time of this point is above 0,
                self.end_times[point.index - 2]
            };
            let max_time = self.end_times[point.index];

            if time <= min_time {
                let value = self.get_point_value(point);
                self.values.remove(point.index);
                self.transitions.remove(point.index - 1);
                self.end_times.remove(point.index - 1);

                self.values[point.index - 1].right_limit = value;

                // ensure first point is continuous
                // it may be that we fuse with the first point
                self.values[0].left_limit = self.values[0].right_limit;

                CurvePointId {
                    index: point.index - 1,
                    side: if point.index == 1 {
                        CurvePointSide::Continuous
                    } else {
                        CurvePointSide::Right
                    }
                }

            } else if time >= max_time {
                let value = self.get_point_value(point);
                self.values.remove(point.index);
                self.transitions.remove(point.index);
                self.end_times.remove(point.index - 1);

                self.values[point.index].left_limit = value;

                // ensure last point is continuous
                // it may be that we fuse with the last point
                let last = self.values.last_mut().unwrap();
                last.right_limit = last.left_limit;

                CurvePointId {
                    index: point.index,
                    side: if point.index == self.values.len() - 1 {
                        CurvePointSide::Continuous
                    } else {
                        CurvePointSide::Left
                    }
                }

            } else {
                self.end_times[point.index - 1] = time;
                point
            }

        } else {
            if point.side == CurvePointSide::Left && time < self.get_point_time(point) {
                let min_time = self.get_point_time(self.prev_point(point).unwrap());
                let value = self.get_point_value(point);

                let val = &mut self.values[point.index];
                val.left_limit = val.right_limit;

                if time <= min_time {
                    self.values[point.index - 1].right_limit = value;
                    self.transitions[point.index - 1] = CurveShape::Linear;

                    // ensure first point is continuous
                    // it may be that we re-fuse with the first point
                    self.values[0].left_limit = self.values[0].right_limit;

                    CurvePointId {
                        index: point.index - 1,
                        side: if point.index == 1 {
                            CurvePointSide::Continuous
                        } else {
                            CurvePointSide::Right
                        }
                    }

                } else {
                    self.values.insert(point.index, CurveYValue::new_single(value));
                    self.transitions.insert(point.index, CurveShape::Linear);
                    self.end_times.insert(point.index - 1, time);

                    CurvePointId {
                        index: point.index,
                        side: CurvePointSide::Continuous,
                    }
                }
            } else if point.side == CurvePointSide::Right && time > self.get_point_time(point) {
                let max_time = self.get_point_time(self.next_point(point).unwrap());
                let value = self.get_point_value(point);

                let val = &mut self.values[point.index];
                val.right_limit = val.left_limit;

                if time >= max_time {
                    self.values[point.index + 1].left_limit = value;
                    self.transitions[point.index] = CurveShape::Linear;

                    // ensure first point is continuous
                    // it may be that we re-fuse with the first point
                    let last = self.values.last_mut().unwrap();
                    last.right_limit = last.left_limit;

                    CurvePointId {
                        index: point.index + 1,
                        side: if point.index + 2 == self.values.len() {
                            CurvePointSide::Continuous
                        } else {
                            CurvePointSide::Left
                        }
                    }

                } else {
                    self.values.insert(point.index + 1, CurveYValue::new_single(value));
                    self.transitions.insert(point.index + 1, CurveShape::Linear);
                    self.end_times.insert(point.index, time);

                    CurvePointId {
                        index: point.index + 1,
                        side: CurvePointSide::Continuous,
                    }
                }
            } else {
                point
            }
        }
    }

    /// sets the shape of the given segment
    pub fn set_segment_shape(&mut self, segment: CurveSegmentId, shape: CurveShape) {
        debug_assert!(self.segment_is_valid(segment), "segment is not contained in the curve");
        self.transitions[segment.index] = shape;
    }

    /// gets the value at the given curve point 
    pub fn get_point_value(&self, point: CurvePointId) -> f64 {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");
        if point.side == CurvePointSide::Right {
            self.values[point.index].right_limit
        } else {
            self.values[point.index].left_limit
        }
    }

    /// gets the time at the given curve point 
    pub fn get_point_time(&self, point: CurvePointId) -> f64 {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");
        if point.index <= 0 {
            0.0
        } else {
            *self.end_times.get(point.index - 1).unwrap()
        }
    }

    /// gets the shape of the given segment
    pub fn get_segment_shape(&self, segment: CurveSegmentId) -> CurveShape {
        debug_assert!(self.segment_is_valid(segment), "segment is not contained in the curve");
        *self.transitions.get(segment.index).unwrap()
    }

    // gets the point starting the segment
    pub fn get_segment_start_point(&self, segment: CurveSegmentId) -> CurvePointId {
        debug_assert!(self.segment_is_valid(segment), "segment is not contained in the curve");
        CurvePointId {
            index: segment.index,
            side: if self.values[segment.index].is_continuous() {
                CurvePointSide::Continuous
            } else {
                CurvePointSide::Right
            }
        }
    }

    // gets the point ending the segment
    pub fn get_segment_end_point(&self, segment: CurveSegmentId) -> CurvePointId {
        debug_assert!(self.segment_is_valid(segment), "segment is not contained in the curve");
        CurvePointId {
            index: segment.index + 1,
            side: if self.values[segment.index + 1].is_continuous() {
                CurvePointSide::Continuous
            } else {
                CurvePointSide::Left
            }
        }
    }

    // gets the curve shape to the left of the point
    pub fn get_point_left_shape(&self, point: CurvePointId) -> Option<CurveShape> {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");
        if self.point_is_start(point) {
            None
        } else {
            Some(self.transitions[point.index - 1])
        }
    }

    // gets the curve shape to the right of the point
    pub fn get_point_right_shape(&self, point: CurvePointId) -> Option<CurveShape> {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");
        if self.point_is_end(point) {
            None
        } else {
            Some(self.transitions[point.index])
        }
    }

    // gets the segment to the left of the point
    pub fn get_point_left_segment(&self, point: CurvePointId) -> Option<CurveSegmentId> {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");
        if self.point_is_start(point) {
            None
        } else {
            Some(CurveSegmentId { index: point.index - 1 })
        }
    }

    // gets the segment to the right of the point
    pub fn get_point_right_segment(&self, point: CurvePointId) -> Option<CurveSegmentId> {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");
        if self.point_is_end(point) {
            None
        } else {
            Some(CurveSegmentId { index: point.index })
        }
    }
    
    // returns the point to the left of the given point
    pub fn prev_point(&self, point: CurvePointId) -> Option<CurvePointId> {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");
        if self.point_is_start(point) {
            None
        } else if point.side == CurvePointSide::Right {
            Some(CurvePointId {
                index: point.index,
                side: CurvePointSide::Left,
            })
        } else {
            Some(CurvePointId {
                index: point.index - 1,
                side: if self.values[point.index - 1].is_continuous() {
                    CurvePointSide::Continuous
                } else {
                    CurvePointSide::Right
                },
            })
        }
    }

    // returns the point to the right of the given point
    pub fn next_point(&self, point: CurvePointId) -> Option<CurvePointId> {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");
        if self.point_is_end(point) {
            None
        } else if point.side == CurvePointSide::Left {
            Some(CurvePointId {
                index: point.index,
                side: CurvePointSide::Right,
            })
        } else {
            Some(CurvePointId {
                index: point.index + 1,
                side: if self.values[point.index + 1].is_continuous() {
                    CurvePointSide::Continuous
                } else {
                    CurvePointSide::Left
                },
            })
        }
    }

    // returns the segment to the left of the given segment
    pub fn prev_segment(&self, segment: CurveSegmentId) -> Option<CurveSegmentId> {
        debug_assert!(self.segment_is_valid(segment), "point is not contained in the curve");
        if self.segment_is_start(segment) {
            None
        } else {
            Some(CurveSegmentId {
                index: segment.index - 1
            })
        }
    }

    // returns the segment to the right of the given segment
    pub fn next_segment(&self, segment: CurveSegmentId) -> Option<CurveSegmentId> {
        debug_assert!(self.segment_is_valid(segment), "point is not contained in the curve");
        if self.segment_is_end(segment) {
            None
        } else {
            Some(CurveSegmentId {
                index: segment.index + 1
            })
        }
    }

    // returns true if the given point is neither the first nor last in the curve
    pub fn point_is_intermediate(&self, point: CurvePointId) -> bool {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");
        point.index > 0 && point.index < self.values.len() - 1
    }

    // returns true if the given point is the first in the curve
    pub fn point_is_start(&self, point: CurvePointId) -> bool {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");
        point.index == 0
    }

    // returns true if the given point is the last in the curve
    pub fn point_is_end(&self, point: CurvePointId) -> bool {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");
        point.index == self.values.len() - 1
    }

    // returns true if the given point is contained in the curve and continuity matches the point
    // if the id is for a discontinuity, but the point is continuous, that is fine
    // but if the id is for a continuous point, but the point is discontinuous, there is a problem
    pub fn point_is_valid(&self, point: CurvePointId) -> bool {
        point.index < self.values.len() &&
        (self.values[point.index].is_continuous() || !point.side.is_continuous())
    }

    // returns true if the given segment is the first in the curve
    pub fn segment_is_start(&self, segment: CurveSegmentId) -> bool {
        debug_assert!(self.segment_is_valid(segment), "segment is not contained in the curve");
        segment.index == 0
    }

    // returns true if the given segment is the last in the curve
    pub fn segment_is_end(&self, segment: CurveSegmentId) -> bool {
        debug_assert!(self.segment_is_valid(segment), "segment is not contained in the curve");
        segment.index == self.transitions.len() - 1
    }

    // returns true if the given segment is contained in the curve
    pub fn segment_is_valid(&self, segment: CurveSegmentId) -> bool {
        segment.index < self.transitions.len()
    }

    // returns the first point in the curve
    pub fn first_point(&self) -> CurvePointId {
        CurvePointId {
            index: 0,
            side: CurvePointSide::Continuous
        }
    }

    // returns the last point in the curve
    pub fn last_point(&self) -> CurvePointId {
        CurvePointId {
            index: self.values.len() - 1,
            side: CurvePointSide::Continuous
        }
    }

    // returns the first segment in the curve
    pub fn first_segment(&self) -> CurveSegmentId {
        CurveSegmentId {
            index: 0,
        }
    }

    // checks if a continuous point contains the given partial point
    // or if two points are equal
    pub fn does_point_contain_partial(&self, point: CurvePointId, partial: CurvePointId) -> bool {
        return point == partial ||
        (point.index == partial.index && self.values[point.index].is_continuous())
    }

    // returns the last segment in the curve
    pub fn last_segment(&self) -> CurveSegmentId {
        CurveSegmentId {
            index: self.transitions.len() - 1,
        }
    }

    // returns  an iterator over the segments in the curve
    pub fn segment_iter(&self) -> CurveSegmentIter {
        CurveSegmentIter {
            curve: self,
            current: Some(self.first_segment()),
        }
    }
 
    // returns an iterator over the points in the curve
    pub fn point_iter(&self) -> CurvePointIter {
        CurvePointIter {
            curve: self,
            current: Some(self.first_point())
        }
    }

    // returns the coordinates of the given point
    pub fn get_point_coords(&self, point: CurvePointId) -> (f64, f64) {
        debug_assert!(self.point_is_valid(point), "point is not contained in the curve");
        (self.get_point_time(point), self.get_point_value(point))
    }

    // returns an iterator over the coordinates of the points in the curve
    pub fn point_coords_iter(&self) -> impl Iterator {
        self.point_iter().map(|f| self.get_point_coords(f))
    }
    
    // returns an iterator over the pairs of points in the curve
    pub fn point_pairs_iter(&self) -> impl Iterator<Item = (CurvePointId, CurvePointId)> {
        self.point_iter().zip(self.point_iter().skip(1))
    }
}

#[derive(Debug, Clone)]
pub struct CurvePointIter<'a> {
    curve: &'a Curve,
    current: Option<CurvePointId>,
}

impl Iterator for CurvePointIter<'_> {
    type Item = CurvePointId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(point) = self.current {
            self.current = self.curve.next_point(point);
            Some(point)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct CurveSegmentIter<'a> {
    curve: &'a Curve,
    current: Option<CurveSegmentId>,
}

impl Iterator for CurveSegmentIter<'_> {
    type Item = CurveSegmentId;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(segment) = self.current {
            self.current = self.curve.next_segment(segment);
            Some(segment)
        } else {
            None
        }
    }
}

