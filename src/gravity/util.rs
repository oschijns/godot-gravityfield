//!
//! Utility functions
//!

/// Convert a symbol -, _ or + into -1.0, 0.0 or 1.0
#[macro_export]
macro_rules! unit {
    ( 0 ) => {
        0.0
    };
    ( _ ) => {
        0.0
    };
    ( - ) => {
        -1.0
    };
    ( + ) => {
        1.0
    };
}

pub mod util2d {

    use crate::gravity::{Field, build_trs::Basis2};
    use godot::{builtin::*, classes::Area2D, global::is_zero_approx, obj::WithBaseField};

    /// Define a minimal 2D vector
    pub const MIN_SIZE: Vector2 = Vector2::new(real::MIN_POSITIVE, real::MIN_POSITIVE);

    /// Get the UP direction in global space based on the local UP direction.
    #[inline]
    pub(crate) fn global_direction<F>(spatial: &F, position: &Vector2) -> Vector2
    where
        F: Field<Vector2> + WithBaseField<Base = Area2D>,
    {
        spatial
            .local_up(position)
            .rotated(spatial.base().get_global_rotation())
    }

    /// Flatten a vector along the X-axis
    #[inline]
    pub fn flatten_x(v: &Vector2) -> Vector2 {
        Vector2::new(0.0, v.y)
    }

    /// Flatten a vector along the Y-axis
    #[inline]
    pub fn flatten_y(v: &Vector2) -> Vector2 {
        Vector2::new(v.x, 0.0)
    }

    /// Return true if the angle between the two vectors is acute
    #[inline]
    pub fn is_acute(a: &Vector2, b: &Vector2) -> bool {
        a.dot(*b) > 0.0
    }

    /// Return true if the angle between the two vectors is a right angle
    #[inline]
    pub fn is_orthogonal(a: &Vector2, b: &Vector2) -> bool {
        is_zero_approx(a.dot(*b) as f64)
    }

    /// Return true if the angle between the two vectors is obtuse
    #[inline]
    pub fn is_obtuse(a: &Vector2, b: &Vector2) -> bool {
        a.dot(*b) < 0.0
    }

    /// Basis axis-aligned orientations for capsule shapes
    pub const ROT_X: Basis2 = [Vector2::new(0.0, 1.0), Vector2::new(-1.0, 0.0)];
    pub const ROT_Y: Basis2 = [Vector2::new(1.0, 0.0), Vector2::new(0.0, 1.0)];
}

pub mod util3d {

    use crate::gravity::{Field, axis::Axis3D};
    use godot::{
        builtin::{math::FloatExt, *},
        classes::Area3D,
        obj::WithBaseField,
    };

    /// Define a minimal 2D vector
    pub const MIN_SIZE: Vector3 =
        Vector3::new(real::MIN_POSITIVE, real::MIN_POSITIVE, real::MIN_POSITIVE);

    /// Get the UP direction in global space based on the local UP direction.
    #[inline]
    pub(crate) fn global_direction<F>(spatial: &F, position: &Vector3) -> Vector3
    where
        F: Field<Vector3> + WithBaseField<Base = Area3D>,
    {
        spatial.base().get_global_basis() * spatial.local_up(position)
    }

    /// Flatten a vector along the X-axis
    #[inline]
    pub fn flatten_x(v: &Vector3) -> Vector3 {
        Vector3::new(0.0, v.y, v.z)
    }

    /// Flatten a vector along the Y-axis
    #[inline]
    pub fn flatten_y(v: &Vector3) -> Vector3 {
        Vector3::new(v.x, 0.0, v.z)
    }

    /// Flatten a vector along the Z-axis
    #[inline]
    pub fn flatten_z(v: &Vector3) -> Vector3 {
        Vector3::new(v.x, v.y, 0.0)
    }

    /// Return true if the angle between the two vectors is acute
    #[inline]
    pub fn is_acute(a: &Vector3, b: &Vector3) -> bool {
        a.dot(*b) > 0.0
    }

    /// Return true if the angle between the two vectors is a right angle
    #[inline]
    pub fn is_orthogonal(a: &Vector3, b: &Vector3) -> bool {
        a.dot(*b).is_zero_approx()
    }

    /// Return true if the angle between the two vectors is obtuse
    #[inline]
    pub fn is_obtuse(a: &Vector3, b: &Vector3) -> bool {
        a.dot(*b) < 0.0
    }

    /// Basis axis-aligned orientations for capsule shapes
    pub const BASIS_X: Basis = axis_aligned_basis(Axis3D::Z, 1);
    pub const BASIS_Y: Basis = Basis::IDENTITY;
    pub const BASIS_Z: Basis = axis_aligned_basis(Axis3D::X, 1);

    /// Build an axis aligned basis from scratch
    const fn axis_aligned_basis(axis: Axis3D, rot: i8) -> Basis {
        // Compute the values of sin and cos,
        // since the basis is axis-aligned, they are either -1, 0, 1.
        let rot = rot.rem_euclid(4);
        let (sin, cos) = match rot {
            0 => (0.0, 1.0),  //   0
            1 => (1.0, 0.0),  //  PI/2
            2 => (0.0, -1.0), //  PI
            3 => (-1.0, 0.0), // -PI/2
            _ => (0.0, 0.0),  // should never happen
        };

        // Select the euler angle
        let rows = match axis {
            Axis3D::X => [
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, cos, -sin),
                Vector3::new(0.0, sin, cos),
            ],
            Axis3D::Y => [
                Vector3::new(cos, 0.0, sin),
                Vector3::new(0.0, 1.0, 0.0),
                Vector3::new(-sin, 0.0, cos),
            ],
            Axis3D::Z => [
                Vector3::new(cos, -sin, 0.0),
                Vector3::new(sin, cos, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            ],
        };
        Basis::from_rows(rows[0], rows[1], rows[2])
    }
}
