//!
//! Generic data structure for building transforms
//!

use godot::builtin::*;
use std::marker::PhantomData;

/// Equivalent to `Basis` but for 2D transforms
pub type Basis2 = [Vector2; 2];

/// Simple alias for `Basis`
pub type Basis3 = Basis;

/// Allow building transforms
#[derive(Debug, Clone, Copy)]
pub struct TransformBuilder<Rot, const R: usize, Pos, const P: usize, Trs> {
    /// Set of rotation matrices
    rotations: [Rot; R],

    /// Set of position vectors
    positions: [Pos; P],

    /// Binding for the expected output type
    phantom: PhantomData<Trs>,
}

/// Build 2D transforms
pub type TransformBuilder2D<const R: usize, const P: usize> =
    TransformBuilder<Basis2, R, Vector2, P, Transform2D>;

/// Build 3D transforms
pub type TransformBuilder3D<const R: usize, const P: usize> =
    TransformBuilder<Basis3, R, Vector3, P, Transform3D>;

impl<Rot, const R: usize, Pos, const P: usize, Trs> TransformBuilder<Rot, R, Pos, P, Trs> {
    /// Create a new builder from raw arrays
    #[inline]
    pub fn new(rotations: [Rot; R], positions: [Pos; P]) -> Self {
        Self {
            rotations,
            positions,
            phantom: PhantomData,
        }
    }
}

impl<const R: usize, const P: usize> TransformBuilder2D<R, P> {
    /// Build a transform
    #[inline]
    pub fn build(&self, index_rot: usize, index_pos: usize) -> Transform2D {
        let rot = self.rotations[index_rot];
        Transform2D::from_cols(rot[0], rot[1], self.positions[index_pos])
    }
}

impl<const R: usize, const P: usize> TransformBuilder3D<R, P> {
    /// Build a transform
    #[inline]
    pub fn build(&self, index_rot: usize, index_pos: usize) -> Transform3D {
        Transform3D::new(self.rotations[index_rot], self.positions[index_pos])
    }
}

/// Implement Default for Transform builder
impl<Rot, const R: usize, Pos, const P: usize, Trs> Default
    for TransformBuilder<Rot, R, Pos, P, Trs>
where
    [Rot; R]: Default,
    [Pos; P]: Default,
{
    /// Create a new builder from raw arrays
    #[inline]
    fn default() -> Self {
        Self {
            rotations: Default::default(),
            positions: Default::default(),
            phantom: Default::default(),
        }
    }
}

/// Trivially transform a builder into a transform
impl From<TransformBuilder2D<1, 1>> for Transform2D {
    fn from(value: TransformBuilder2D<1, 1>) -> Self {
        value.build(0, 0)
    }
}

/// Trivially transform a builder into a transform
impl From<TransformBuilder3D<1, 1>> for Transform3D {
    fn from(value: TransformBuilder3D<1, 1>) -> Self {
        value.build(0, 0)
    }
}
