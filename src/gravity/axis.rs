//!
//! Helpers for handling 2D gravity
//!

use godot::prelude::*;

/// Select an axis in 2D space
#[repr(C)]
#[derive(GodotConvert, Var, Export, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[godot(via = GString)]
pub enum Axis2D {
    /// X-axis
    X,

    /// Y-axis
    Y,
}

/// Select an axis in 3D space
#[repr(C)]
#[derive(GodotConvert, Var, Export, Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[godot(via = GString)]
pub enum Axis3D {
    /// X-axis
    X,

    /// Y-axis
    Y,

    /// Z-axis
    Z,
}

impl Axis2D {
    /// To vector
    pub fn to_vector(self) -> Vector2 {
        match self {
            Self::X => Vector2::RIGHT,
            Self::Y => Vector2::UP,
        }
    }
}

impl Axis3D {
    /// To vector
    pub fn to_vector(self) -> Vector3 {
        match self {
            Self::X => Vector3::RIGHT,
            Self::Y => Vector3::UP,
            Self::Z => Vector3::FORWARD,
        }
    }
}

/// Convert selected axis into godot-rust axis type
impl From<Axis2D> for Vector2Axis {
    fn from(value: Axis2D) -> Self {
        match value {
            Axis2D::X => Self::X,
            Axis2D::Y => Self::Y,
        }
    }
}

/// Convert selected axis into godot-rust axis type
impl From<Axis3D> for Vector3Axis {
    fn from(value: Axis3D) -> Self {
        match value {
            Axis3D::X => Self::X,
            Axis3D::Y => Self::Y,
            Axis3D::Z => Self::Z,
        }
    }
}

/// Convert godot-rust axis into selected axis type
impl From<Vector2Axis> for Axis2D {
    fn from(value: Vector2Axis) -> Self {
        match value {
            Vector2Axis::X => Self::X,
            Vector2Axis::Y => Self::Y,
        }
    }
}

/// Convert godot-rust axis into selected axis type
impl From<Vector3Axis> for Axis3D {
    fn from(value: Vector3Axis) -> Self {
        match value {
            Vector3Axis::X => Self::X,
            Vector3Axis::Y => Self::Y,
            Vector3Axis::Z => Self::Z,
        }
    }
}
