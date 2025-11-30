use crate::{live_plugin_id::LivePluginId, playback::{InputId, InputSpecification}, sequencers::curve::{Curve, CurvePointId, CurveSegmentId, CurveShape}};

#[derive(Debug)]
pub enum AutomationId {
    Component(LivePluginId, InputId),
    Tempo
}

#[derive(Debug)]
pub struct AutomationSequencer {
    /// the curve representing the automation
    /// Invariants:
    /// 	1) total_duration == 1
    curve: Curve,
    spec: InputSpecification,
    id: AutomationId,
    duration: f64
}

impl AutomationSequencer {
    /// returns the value at the given time
    /// NOTE: if time is ZERO OR LESS, it will return the first value in the curve
    /// if time is greater than what the curve covers, it will return the last value in the curve
    /// O(log n)
    pub fn value_at_time(&self, time: f64) -> f64 {
        self.curve.value_at_time(time / self.duration)
    }

    /// returns the total duration of the automation
    pub fn total_duration(&self) -> f64 {
        self.duration
    }

    /// returns the segment at the given time
    /// if the time lies on a point, defaults to the right side of the point
    /// if the time is less than 0, defaults to the first segment
    /// if the time is greater than or equal to the total duration, defaults to the last segment
    pub fn get_segment(&self, time: f64) -> CurveSegmentId {
        self.curve.get_segment(time / self.duration)
    }

    /// returns the point nearest to the given time
    pub fn get_nearest_point(&self, time: f64) -> CurvePointId {
        self.curve.get_nearest_point(time / self.duration)
    }

    /// deletes all segments between the given points (inclusive) and the following transition
    ///
    ///	fails if we attempt to delete either the start or end points
    ///
    ///	returns true upon a successful deletion
    pub fn remove_point_to_point(&mut self, point1: CurvePointId, point2: CurvePointId) -> bool {
        if self.curve.point_is_start(point1) ||
        self.curve.point_is_end(point1) ||
        self.curve.point_is_start(point2) ||
        self.curve.point_is_end(point2) {
            false
        } else {
            self.curve.remove_point_to_point(point1, point2);
            true
        }
    }

    /// deletes the given point on the curve and the following transition
    ///
    /// fails if we attempt to delete either the start or end point
    ///
    /// returns true upon a successful deletion
    pub fn remove_point(&mut self, point: CurvePointId) -> bool {
        if self.curve.point_is_start(point) || self.curve.point_is_end(point) {
            false
        } else {
            self.curve.remove_point(point);
            true
        }
    }

    /// adds a point at the given time
    ///
    /// the point will be placed such that it is on the existing curve. 
    /// it will also take on the transitions it broke up on either side of it
    ///
    /// fails if we attempt to add before/at 0.0 or after total_duration
    /// or if we attempt to add at an already existing point
    ///
    /// returns true if the new point was added
    pub fn add_point(&mut self, time: f64) -> Option<CurvePointId> {
        if time <= 0.0 || time >= self.duration {
            None
        } else {
            self.curve.add_point(time / self.duration)
        }
    }

    /// sets the value of the given point, clamped to the input's range
    pub fn set_point_value(&mut self, point: CurvePointId, value: f64) {
        self.curve.set_point_value(point, value.clamp(self.spec.range.0, self.spec.range.1));
    }

    /// moves the time of the given point
    /// 
    /// fails if you try to move either the start or end points
    ///
    /// returns true upon a succesful operation
    pub fn set_point_time(&mut self, point: CurvePointId, time: f64) -> bool {
        if self.curve.point_is_start(point) || self.curve.point_is_end(point) {
            false
        } else {
            self.curve.set_point_time(point, time / self.duration);
            true
        }
    }

    /// sets the shape of the given segment
    pub fn set_segment_shape(&mut self, segment: CurveSegmentId, shape: CurveShape) {
        self.curve.set_segment_shape(segment, shape);
    }
}
