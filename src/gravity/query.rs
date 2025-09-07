//!
//! Define a resource for querying the physics engine for gravity direction
//!

/// Simple macro to quicly implement both gravity queries type
macro_rules! gravity_query {
    (
        $query_type:ident where {
            [$flag:ident],
            $parameters:ty,
            $space:ty,
            $dynamic_type:ident as {
                $area:ty,
                $vector:ty
            }
        }
    ) => {
        /// Dynamic gravity field type
        pub type $dynamic_type = DynGd<$area, dyn Field<$vector>>;

        // alias provided types
        type Dynamic = $dynamic_type;
        type Parameters = $parameters;
        type Space = $space;
        type Vector = $vector;

        /// Define a gravity query object
        #[derive(GodotClass)]
        #[class(base=Resource)]
        pub struct $query_type {
            base: Base<Resource>,

            /// Internal
            internal: Gd<Parameters>,

            /// Define the collision mask
            #[export($flag)]
            #[var(get, set = set_collision_mask)]
            collision_mask: Mask,

            /// Define the maximum number of results to report
            #[export(range = (0.0, 1.0, or_greater))]
            max_results: u32,
        }

        #[godot_api]
        impl IResource for $query_type {
            /// Instantiate the resource
            fn init(base: Base<Resource>) -> Self {
                const MASK: Mask = 0b1;

                // instantiate a parameters object
                let mut internal = Parameters::new_gd();
                internal.set_collision_mask(MASK);
                internal.set_collide_with_bodies(false);
                internal.set_collide_with_areas(true);

                Self {
                    base,
                    internal,
                    collision_mask: MASK,
                    max_results: 32,
                }
            }
        }

        impl $query_type {
            pub fn gravity_direction(
                &self,
                space: &mut Space,
                position: &Vector,
            ) -> Option<(Vector, Vec<Dynamic>)> {
                // prepare the parameters
                let mut params = self.internal.clone();
                params.set_position(*position);

                // perform the physics query
                let results = space
                    .intersect_point_ex(&params)
                    .max_results(self.max_results as i32)
                    .done();

                // look up the results to identify the gravity fields to use
                if results.is_empty() {
                    None
                } else {
                    // try to find the best gravity fields
                    let mut level = Level::MIN;
                    let mut up = Vector::ZERO;
                    let mut fields = Vec::new();

                    // check each gravity field found
                    for result in results.iter_shared() {
                        // Check if the point is a Gravity Field
                        if let Ok(area) = Dynamic::try_from_variant(&result.get_or_nil("collider"))
                        {
                            // get access to the gravity field trait
                            let field = area.dyn_bind();
                            let new_level = field.level();

                            // Based on the level of the gravity field, either
                            // reset the current list, simply add it or ignore it.
                            if new_level > level {
                                level = new_level;
                                up = field.global_up(position);
                                fields.clear();
                                fields.push(area.clone());
                            } else if new_level == level {
                                up += field.global_up(position);
                                fields.push(area.clone());
                            }
                        }
                    }

                    Some((up.normalized_or_zero(), fields))
                }
            }
        }

        #[godot_api]
        impl $query_type {
            #[func]
            pub fn find_gravity_direction(
                &self,
                mut space: Gd<Space>,
                position: Vector,
            ) -> Dictionary {
                if let Some((up, fields)) = self.gravity_direction(space.deref_mut(), &position) {
                    vdict! {
                        "up": up.to_variant(),
                        "fields": fields.to_variant()
                    }
                } else {
                    Dictionary::new()
                }
            }

            #[func]
            #[inline]
            pub fn set_collision_mask(&mut self, collision_mask: Mask) {
                self.collision_mask = collision_mask;
            }
        }
    };
}

pub mod inner2d {
    use crate::gravity::{Field, Level, Mask};
    use godot::{
        classes::{Area2D, PhysicsDirectSpaceState2D, PhysicsPointQueryParameters2D, Resource},
        prelude::*,
    };
    use std::ops::DerefMut;

    gravity_query! {
        GravityQuery2D where {
            [flags_2d_physics],
            PhysicsPointQueryParameters2D,
            PhysicsDirectSpaceState2D,
            DynGravityField2D as {
                Area2D,
                Vector2
            }
        }
    }
}

pub mod inner3d {
    use crate::gravity::{Field, Level, Mask};
    use godot::{
        classes::{Area3D, PhysicsDirectSpaceState3D, PhysicsPointQueryParameters3D, Resource},
        prelude::*,
    };
    use std::ops::DerefMut;

    gravity_query! {
        GravityQuery3D where {
            [flags_3d_physics],
            PhysicsPointQueryParameters3D,
            PhysicsDirectSpaceState3D,
            DynGravityField3D as {
                Area3D,
                Vector3
            }
        }
    }
}

// re-export types
pub use inner2d::{DynGravityField2D, GravityQuery2D};
pub use inner3d::{DynGravityField3D, GravityQuery3D};
