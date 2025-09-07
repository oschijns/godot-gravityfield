//!
//! Gravity field backed by a shape resource
//!

/// Define a cuboid shape
pub mod cuboid;

/// Define a shaped backed by a curve
pub mod curve;

use godot::obj::{Gd, GodotClass};

/// Trait to implement a shape for a gravity field
pub trait Shape<V, Shp, Trs>
where
    Shp: GodotClass,
{
    /// Get the UP direction for the given position.
    fn up(&self, position: &V) -> V;

    /// Get the list of colliders to generate a static body.
    fn colliders(&mut self) -> Vec<(Gd<Shp>, Trs)>;
}

/// Interface for internal shape representation
pub trait MakeColliders<Shp, Trs>
where
    Shp: GodotClass,
{
    /// Get the list of colliders to generate a static body.
    fn colliders(&self) -> Vec<(Gd<Shp>, Trs)>;
}

macro_rules! gravity_field_shaped {
    (
        $shape_type:ty where {
            $area:ty | $area_interface:ty,
            $vector:ty,
            $dynamic_type:ident
        }
    ) => {
        // alias provided type
        type Vector = $vector;
        type Area = $area;

        #[godot_api]
        impl $area_interface for $shape_type {
            /// Instantiate the node
            fn init(base: Base<Area>) -> Self {
                Self {
                    base,
                    level: 0,
                    shape: None,
                    build_collider: false,
                    inverted: false,
                }
            }
        }

        impl Field<Vector> for $shape_type {
            /// Get the priority level
            #[inline]
            fn level(&self) -> Level {
                self.level
            }

            /// Up direction is solely defined by the axis selected
            fn local_up(&self, _position: &Vector) -> Vector {
                let up = Vector::ZERO;

                // Check if the direction should be inverted
                if self.inverted { -up } else { up }
            }

            /// Up direction is solely defined by the axis selected
            fn global_up(&self, position: &Vector) -> Vector {
                global_direction(self, position)
            }
        }

        #[godot_api]
        impl $shape_type {
            #[func]
            pub fn get_up_direction(&self, position: Vector) -> Vector {
                self.global_up(&position)
            }

            #[func]
            fn set_build_collider(&mut self, set: bool) {
                self.build_collider = set;
            }
        }
    };
}

pub mod inner2d {

    use super::Shape;
    use crate::gravity::{Field, Level, util::util2d::global_direction};
    use godot::{
        classes::{Area2D, IArea2D, Resource, Shape2D},
        obj::DynGd,
        prelude::*,
    };

    /// Dynamic shape type
    pub type DynShape2D = DynGd<Resource, dyn Shape<Vector2, Shape2D, Transform2D>>;

    /// Define a gravity based on a supporting shape.
    #[derive(GodotClass)]
    #[class(base=Area2D)]
    pub struct GravityShaped2D {
        base: Base<Area2D>,

        /// Priority level
        #[export]
        level: Level,

        /// The shape definition to use
        #[export]
        shape: Option<DynShape2D>,

        /// Specify if a static body should be generated
        #[export]
        #[var(get, set = set_build_collider)]
        build_collider: bool,

        /// Inverse the gravity
        #[export]
        inverted: bool,
    }

    gravity_field_shaped! {
        GravityShaped2D where {
            Area2D | IArea2D,
            Vector2,
            DynShape2D
        }
    }
}

pub mod inner3d {

    use super::Shape;
    use crate::gravity::{Field, Level, util::util3d::global_direction};
    use godot::{
        classes::{Area3D, IArea3D, Resource, Shape3D},
        obj::DynGd,
        prelude::*,
    };

    /// Dynamic shape type
    pub type DynShape3D = DynGd<Resource, dyn Shape<Vector3, Shape3D, Transform3D>>;

    /// Define a gravity based on a supporting shape.
    #[derive(GodotClass)]
    #[class(base=Area3D)]
    pub struct GravityShaped3D {
        base: Base<Area3D>,

        /// Priority level
        #[export]
        level: Level,

        /// The shape definition to use
        #[export]
        shape: Option<DynShape3D>,

        /// Specify if a static body should be generated
        #[export]
        #[var(get, set = set_build_collider)]
        build_collider: bool,

        /// Inverse the gravity
        #[export]
        inverted: bool,
    }

    gravity_field_shaped! {
        GravityShaped3D where {
            Area3D | IArea3D,
            Vector3,
            DynShape3D
        }
    }
}

// re-export
pub use inner2d::{DynShape2D, GravityShaped2D};
pub use inner3d::{DynShape3D, GravityShaped3D};
