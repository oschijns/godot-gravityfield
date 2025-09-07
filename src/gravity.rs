//!
//! Module providing components usable in both 2D and 3D variants
//!

/// Define query
pub mod query;

/// Define axises
pub mod axis;

/// Generic data structure for building transforms
pub mod build_trs;

/// Define gravity fields
pub mod field;

/// Utility functions
pub mod util;

/// Type used to define priority level
pub type Level = i32;

/// Type used to define collision masks
pub type Mask = u32;

/// Trait to implement a gravity field
pub trait Field<V> {
    /// Get the priority level of the gravity field
    fn level(&self) -> Level;

    /// Get the UP direction for the given position in the local space of the gravity field.
    fn local_up(&self, position: &V) -> V;

    /// Get the UP direction for the given position in global space.
    fn global_up(&self, position: &V) -> V;
}

#[macro_export]
macro_rules! export_gravity_up {
    ( $gravity_field_type:ty => $vector:ty ) => {
        #[godot_api]
        impl $gravity_field_type {
            #[func]
            pub fn get_up_direction(&self, position: $vector) -> $vector {
                self.global_up(&position)
            }
        }
    };
}
