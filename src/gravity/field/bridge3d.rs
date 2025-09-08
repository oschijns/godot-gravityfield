//!
//! Define bridge gravity fields
//!

use crate::{
    export_gravity_up,
    gravity::{Field, Level},
};
use godot::{
    classes::{Area3D, IArea3D},
    prelude::*,
};

/// Define a smooth transition between multiple other gravity fields.
#[derive(GodotClass)]
#[class(base=Area3D)]
pub struct GravityBridge3D {
    base: Base<Area3D>,

    /// Priority level
    #[export]
    level: Level,

    /// List of points to pull from
    points: Vec<BridgePoint>,
}

/// Other gravity field used to evaluate
pub struct BridgePoint {
    /// The gravity field to pull from
    field: DynGd<Area3D, dyn Field<Vector3>>,

    /// Delimitation to pull fully from the gravity field.
    /// The plane is define in the local space of the associated field.
    /// If the point is over the field the UP direction will smoothly
    /// transition with the UP direction of the other fields.
    local_plane: Plane,
}

#[godot_api]
impl IArea3D for GravityBridge3D {
    /// Instantiate the node
    fn init(base: Base<Area3D>) -> Self {
        Self {
            base,
            level: 0,
            points: Vec::new(),
        }
    }
}

export_gravity_up![GravityBridge3D => Vector3];

impl Field<Vector3> for GravityBridge3D {
    /// Get the priority level
    #[inline]
    fn level(&self) -> Level {
        self.level
    }

    /// Local UP direction is evaluated from the global UP direction this time.
    fn local_up(&self, position: &Vector3) -> Vector3 {
        self.base().get_global_basis().inverse() * self.global_up(position)
    }

    /// Pull a direction from the various gravity fields referenced.
    fn global_up(&self, position: &Vector3) -> Vector3 {
        if self.points.is_empty() {
            Vector3::ZERO
        } else {
            let count = self.points.len();

            // Prepare to lists of fields where the object is over the delimitation plane
            // and another list for fields where the object is below the delimitation plane.
            let mut above = Vec::with_capacity(count);
            let mut below = Vec::with_capacity(count);

            // Evaluate the up direction for each gravity field
            for point in self.points.iter() {
                let plane = point.get_global_plane();

                if plane.is_point_over(*position) {
                    // Project the position onto the plane.
                    // Use that position to get the UP direction.
                    let projected = plane.project(*position);
                    let direction = *position - projected;
                    let up = point.field.dyn_bind().global_up(&projected);

                    // Also compute the distance from the plane.
                    let distance = plane.distance_to(*position);
                    above.push(InsideData::new(up, direction, distance));
                } else {
                    // Get the UP direction and the distance from the plane.
                    let up = point.field.dyn_bind().global_up(position);
                    let distance = plane.distance_to(*position);
                    below.push(OutsideData::new(up, distance));
                }
            }

            // Pick the list to use for interpolation
            if below.is_empty() {
                // Evaluate the UP direction inside of the convex shape.
                if above.len() > 1 {
                    // Compute a Weighted Spherical Linear Interpolation.

                    // First compute the Mean Value Coordinates of each point.
                    // Compute the tangent of the half angle between two consecutive pooled points.
                    let last = above.len() - 1;
                    for index in 0..last {
                        let translation = above[index + 1].translation;
                        above[index].compute_tangent_angle(translation);
                    }
                    let translation = above[0].translation;
                    above[last].compute_tangent_angle(translation);

                    // Compute the weight between two consecutive pooled points.
                    for index in 1..above.len() {
                        let prev_tan = above[index - 1].tan_angle;
                        above[index].compute_weigth(prev_tan);
                    }
                    let prev_tan = above[last].tan_angle;
                    above[last].compute_weigth(prev_tan);

                    // At that point the sum of all the weights should be 1.

                    // Now we can blend the UP directions.
                    let mut sum_up = Vector3::ZERO;
                    for data in above.iter() {
                        sum_up += data.up * data.weight;
                    }
                    sum_up.normalized_or_zero()
                } else {
                    // Only one point, return it directly.
                    above[0].up
                }
            } else {
                // The position is outside of the convex shape.
                // Fallback to a less accurate evaluation of the UP direction.
                if below.len() > 1 {
                    // Find the proper interpolation using Distance-based Weighted Averaging.
                    // The result may not be perfect but it is simple and fast.

                    // Sum the inverse of the distance to evaluate the denominator.
                    let mut denominator = 0.0;
                    for data in below.iter_mut() {
                        data.weight = 1.0 / data.distance;
                        denominator += data.weight;
                    }

                    // Weight in the range [0, 1] to increment at every step.
                    let mut sum_up = Vector3::ZERO;

                    // Now perform the interpolations
                    for data in below.iter() {
                        sum_up += data.up * (data.weight / denominator);
                    }
                    sum_up.normalized_or_zero()
                } else {
                    // With only one element, pick the UP direction as is.
                    below[0].up
                }
            }
        }
    }
}

impl BridgePoint {
    #[inline]
    fn get_global_plane(&self) -> Plane {
        self.field.get_global_transform() * self.local_plane
    }
}

/// Data for UP direction computation when
/// the position is inside of the convex shape.
struct InsideData {
    /// UP direction computed by projecting the point onto the convex shape's surface.
    up: Vector3,

    /// Translation from the shape's surface toward the point.
    translation: Vector3,

    /// Distance between the shape's surface and the point.
    distance: real,

    /// Half of the angle between two translations of consecutive "InsideData".
    tan_angle: real,

    /// Weight computed from two consecutive "InsideData".
    weight: real,
}

impl InsideData {
    fn new(up: Vector3, translation: Vector3, distance: real) -> Self {
        Self {
            up,
            translation,
            distance,
            tan_angle: 0.0,
            weight: 0.0,
        }
    }

    #[inline]
    fn compute_tangent_angle(&mut self, next_direction: Vector3) {
        let angle = self.translation.angle_to(next_direction);
        self.tan_angle = (angle * 0.5).tan();
    }

    #[inline]
    fn compute_weigth(&mut self, prev_tan_angle: real) {
        self.weight = (prev_tan_angle + self.tan_angle) / self.distance;
    }
}

/// Data for UP direction computation when
/// the position is outside of the convex shape.
struct OutsideData {
    /// UP direction computed at the point.
    up: Vector3,

    /// Distance between the point and the surface.
    distance: real,

    /// Distance-based weight.
    weight: real,
}

impl OutsideData {
    fn new(up: Vector3, distance: real) -> Self {
        Self {
            up,
            distance,
            weight: 0.0,
        }
    }
}
