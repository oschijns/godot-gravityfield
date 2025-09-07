//!
//! Define center gravity fields
//!

macro_rules! gravity_field_center {
    (
        $field_type:ty where {
            $area:ty | $area_interface:ty,
            $vector:ty
        }
    ) => {
        // alias provided types
        type Area = $area;
        type Vector = $vector;

        #[godot_api]
        impl $area_interface for $field_type {
            /// Instantiate the node
            fn init(base: Base<Area>) -> Self {
                Self {
                    base,
                    level: 0,
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

            /// Up direction is defined by the relative direction of the object.
            fn local_up(&self, position: &Vector) -> Vector {
                let up = position.normalized_or_zero();

                // Check if the direction should be inverted
                if self.inverted { -up } else { up }
            }

            /// Up direction is defined by the relative direction of the object.
            fn global_up(&self, position: &Vector) -> Vector {
                global_direction(self, position)
            }
        }
    };
}

pub mod inner2d {

    use crate::{
        export_gravity_up,
        gravity::{Field, Level, util::util2d::global_direction},
    };
    use godot::{
        classes::{Area2D, IArea2D},
        prelude::*,
    };

    /// Define a gravity centered around a point.
    #[derive(GodotClass)]
    #[class(base=Area2D)]
    pub struct GravityCenter2D {
        base: Base<Area2D>,

        /// Priority level
        #[export]
        level: Level,

        /// Inverse the gravity
        #[export]
        inverted: bool,
    }

    export_gravity_up![GravityCenter2D => Vector2];

    gravity_field_center! {
        GravityCenter2D where {
            Area2D | IArea2D,
            Vector2
        }
    }
}

pub mod inner3d {

    use crate::{
        export_gravity_up,
        gravity::{Field, Level, util::util3d::global_direction},
    };
    use godot::{
        classes::{Area3D, IArea3D},
        prelude::*,
    };

    /// Define a gravity centered around a point.
    #[derive(GodotClass)]
    #[class(base=Area3D)]
    pub struct GravityCenter3D {
        base: Base<Area3D>,

        /// Priority level
        #[export]
        level: Level,

        /// Inverse the gravity
        #[export]
        inverted: bool,
    }

    export_gravity_up![GravityCenter3D => Vector3];

    gravity_field_center! {
        GravityCenter3D where {
            Area3D | IArea3D,
            Vector3
        }
    }
}

// re-export types
pub use inner2d::GravityCenter2D;
pub use inner3d::GravityCenter3D;
