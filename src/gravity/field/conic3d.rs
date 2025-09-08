//!
//! Define conic gravity fields
//!

use crate::{
    export_gravity_up,
    gravity::{
        Field, Level,
        axis::Axis3D,
        util::util3d::{flatten_x, flatten_y, flatten_z, global_direction},
    },
};
use godot::{
    classes::{Area3D, IArea3D},
    prelude::*,
};

/// Define a gravity centered around a cone shape.
#[derive(GodotClass)]
#[class(base=Area3D)]
pub struct GravityConic3D {
    base: Base<Area3D>,

    /// Priority level
    #[export]
    level: Level,

    /// The height of the cone
    #[export(range = (0.0, 10.0, or_greater))]
    height: real,

    /// The radius of the cone
    #[export(range = (0.0, 1.0, or_greater))]
    radius: real,

    /// Central Axis
    #[export]
    axis: Axis3D,

    /// Inverse the gravity
    #[export]
    inverted: bool,
}

#[godot_api]
impl IArea3D for GravityConic3D {
    /// Instantiate the node
    fn init(base: Base<Area3D>) -> Self {
        Self {
            base,
            level: 0,
            height: 1.0,
            radius: 0.5,
            axis: Axis3D::Y,
            inverted: false,
        }
    }
}

export_gravity_up![GravityConic3D => Vector3];

impl Field<Vector3> for GravityConic3D {
    /// Get the priority level
    #[inline]
    fn level(&self) -> Level {
        self.level
    }

    /// Up direction is defined by the relative position
    /// of the object around the selected axis.
    fn local_up(&self, position: &Vector3) -> Vector3 {
        // Pick the up direction based on the axis selected
        let up = match self.axis {
            Axis3D::X => {
                let mut v = flatten_x(position);
                let len = v.length();
                v.x = self.radius * len;
                v.y *= self.height;
                v.z *= self.height;
                v.normalized_or_zero()
            }
            Axis3D::Y => {
                let mut v = flatten_y(position);
                let len = v.length();
                v.x *= self.height;
                v.y = self.radius * len;
                v.z *= self.height;
                v.normalized_or_zero()
            }
            Axis3D::Z => {
                let mut v = flatten_z(position);
                let len = v.length();
                v.x *= self.height;
                v.y *= self.height;
                v.z = self.radius * len;
                v.normalized_or_zero()
            }
        };

        // Check if the direction should be inverted
        if self.inverted { -up } else { up }
    }

    /// Up direction is defined by the relative position
    /// of the object around the selected axis.
    fn global_up(&self, position: &Vector3) -> Vector3 {
        global_direction(self, position)
    }
}
