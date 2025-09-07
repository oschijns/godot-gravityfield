//!
//! Flat gravity field
//!

macro_rules! gravity_field_flat {
    (
        $field_type:ty where {
            $area:ty | $area_interface:ty,
            $vector:ty,
            $axis:ty
        }
    ) => {
        // alias provided types
        type Area = $area;
        type Axis = $axis;
        type Vector = $vector;

        #[godot_api]
        impl $area_interface for $field_type {
            /// Instantiate the node
            fn init(base: Base<Area>) -> Self {
                Self {
                    base,
                    level: 0,
                    axis: Axis::Y,
                    inverted: false,
                }
            }
        }

        impl Field<Vector> for $field_type {
            /// Get the priority level
            #[inline]
            fn level(&self) -> Level {
                self.level
            }

            /// Up direction is solely defined by the axis selected
            fn local_up(&self, _position: &Vector) -> Vector {
                // Pick the up direction based on the axis selected
                let up = self.axis.to_vector();

                // Check if the direction should be inverted
                if self.inverted { -up } else { up }
            }

            /// Up direction is solely defined by the axis selected
            fn global_up(&self, position: &Vector) -> Vector {
                global_direction(self, position)
            }
        }
    };
}

pub mod inner2d {

    use crate::{
        export_gravity_up,
        gravity::{Field, Level, axis::Axis2D, util::util2d::global_direction},
    };
    use godot::{
        classes::{Area2D, IArea2D},
        prelude::*,
    };

    /// Define a gravity based on an axis direction.
    #[derive(GodotClass)]
    #[class(base=Area2D)]
    pub struct GravityFlat2D {
        base: Base<Area2D>,

        /// Priority level
        #[export]
        level: Level,

        /// Central Axis
        #[export]
        axis: Axis2D,

        /// Inverse the gravity
        #[export]
        inverted: bool,
    }

    export_gravity_up![GravityFlat2D => Vector2];

    gravity_field_flat! {
        GravityFlat2D where {
            Area2D | IArea2D,
            Vector2,
            Axis2D
        }
    }
}

pub mod inner3d {

    use crate::{
        export_gravity_up,
        gravity::{Field, Level, axis::Axis3D, util::util3d::global_direction},
    };
    use godot::{
        classes::{Area3D, IArea3D},
        prelude::*,
    };

    /// Define a gravity based on an axis direction.
    #[derive(GodotClass)]
    #[class(base=Area3D)]
    pub struct GravityFlat3D {
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

    export_gravity_up![GravityFlat3D => Vector3];

    gravity_field_flat! {
        GravityFlat3D where {
            Area3D | IArea3D,
            Vector3,
            Axis3D
        }
    }
}

// re-export types
pub use inner2d::GravityFlat2D;
pub use inner3d::GravityFlat3D;
