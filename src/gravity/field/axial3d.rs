//!
//! Define axial gravity fields
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

/// Define a gravity centered around an axis.
#[derive(GodotClass)]
#[class(base=Area3D)]
pub struct GravityAxial3D {
    base: Base<Area3D>,

    /// Priority level
    #[export]
    level: Level,

    /// Central Axis
    #[export]
    axis: Axis3D,

    /// Inverse the gravity
    #[export]
    inverted: bool,
}

#[godot_api]
impl IArea3D for GravityAxial3D {
    /// Instantiate the node
    fn init(base: Base<Area3D>) -> Self {
        Self {
            base,
            level: 0,
            axis: Axis3D::Y,
            inverted: false,
        }
    }
}

export_gravity_up![GravityAxial3D => Vector3];

impl Field<Vector3> for GravityAxial3D {
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
            Axis3D::X => flatten_x(position).normalized_or_zero(),
            Axis3D::Y => flatten_y(position).normalized_or_zero(),
            Axis3D::Z => flatten_z(position).normalized_or_zero(),
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
