//!
//! Define a curve resource for shape backed gravity fields
//!

/// Simple macro to prepare 2D and 3D cuboid shapes
macro_rules! shape_curve {
    (
        $shape_type:ident where {
            $transform:ty,
            $vector:ty,
            $curve:ty,
            $shape:ty as $capsule:ty
        }
    ) => {
        // alias provided types
        type Transform = $transform;
        type Vector = $vector;
        type GCurve = $curve;
        type GShape = $shape;
        type Capsule = $capsule;

        /// Define a gravity based on an axis direction.
        #[derive(GodotClass)]
        #[class(base=Resource)]
        pub struct $shape_type {
            base: Base<Resource>,

            /// Generated shapes
            internal: Option<Internal>,

            /// Curve to follow
            #[export]
            #[var(get, set = set_curve)]
            curve: Option<Gd<GCurve>>,

            /// Radius of the curve
            #[export(range = (0.0, 10.0, or_greater))]
            #[var(get, set = set_radius)]
            radius: real,
        }

        #[godot_api]
        impl IResource for $shape_type {
            fn init(base: Base<Resource>) -> Self {
                Self {
                    base,
                    internal: None,
                    curve: None,
                    radius: 0.0,
                }
            }
        }

        #[godot_api]
        impl $shape_type {
            #[func]
            fn set_curve(&mut self, curve: Option<Gd<GCurve>>) {
                self.curve = curve;
                self.internal = None;
            }

            #[func]
            fn set_radius(&mut self, radius: real) {
                self.radius = radius;
                self.internal = None;
            }
        }

        impl Shape<Vector, GShape, Transform> for $shape_type {
            /// Pick the UP direction for a cuboid
            #[inline]
            fn up(&self, position: &Vector) -> Vector {
                if let Some(curve) = &self.curve {
                    curve.get_closest_point(*position).direction_to(*position)
                } else {
                    Vector::ZERO
                }
            }

            /// Return a list of colliders
            fn colliders(&mut self) -> Vec<(Gd<GShape>, Transform)> {
                if let Some(curve) = &self.curve {
                    // Recompute the internal shapes if requested
                    if self.internal.is_none() {
                        self.internal = Some(Internal::new(&curve, self.radius));
                    }

                    // Ask the internal shape for its colliders set
                    self.internal.as_ref().unwrap().colliders()
                } else {
                    Vec::new()
                }
            }
        }

        struct Internal {
            /// Shape used to cover the whole curve
            shape: Gd<Capsule>,

            /// Position the capsule to cover the whole curve
            transforms: Vec<Transform>,
        }

        impl Internal {
            /// Build collision shapes for the curve
            fn new(curve: &GCurve, radius: real) -> Self {
                let points = curve.get_baked_points();

                // build the transforms to create the curve
                let mut transforms = Vec::with_capacity(points.len());

                // iterate over the points two by two
                let last = points.len() - 2;
                for i in 0..last {
                    let p0 = points[i];
                    let p1 = points[i + 1];

                    // create a new transform to position a capsule
                    transforms.push(orient(p0.direction_to(p1), (p0 + p1) * 0.5));
                }

                // Create a capsule shape
                let mut shape = Capsule::new_gd();
                shape.set_radius(radius);
                shape.set_height(curve.get_bake_interval() + radius * 2.0);

                Self { shape, transforms }
            }

            /// Return a list of colliders
            fn colliders(&self) -> Vec<(Gd<GShape>, Transform)> {
                let shape = self.shape.clone().upcast::<GShape>();

                // Create a list
                let mut list = Vec::with_capacity(self.transforms.len());
                for trs in &self.transforms {
                    list.push((shape.clone(), *trs));
                }
                list
            }
        }
    };
}

pub mod inner2d {

    use crate::gravity::field::shaped::Shape;
    use godot::{
        classes::{CapsuleShape2D, Curve2D, Shape2D},
        prelude::*,
    };

    shape_curve! {
        GravityShapedCurve2D where {
            Transform2D,
            Vector2,
            Curve2D,
            Shape2D as CapsuleShape2D
        }
    }

    /// Orient a basis such that its Y-axis point toward the provided direction.
    #[inline]
    fn orient(direction: Vector2, center: Vector2) -> Transform2D {
        Transform2D::from_angle_origin(direction.angle(), center)
    }
}

pub mod inner3d {

    use crate::gravity::field::shaped::Shape;
    use godot::{
        classes::{CapsuleShape3D, Curve3D, Shape3D},
        prelude::*,
    };

    shape_curve! {
        GravityShapedCurve3D where {
            Transform3D,
            Vector3,
            Curve3D,
            Shape3D as CapsuleShape3D
        }
    }

    /// Orient a basis such that its Y-axis point toward the provided direction.
    #[inline]
    fn orient(direction: Vector3, center: Vector3) -> Transform3D {
        // Check if direction is colinear with the up direction
        if direction.x.is_zero_approx() && direction.z.is_zero_approx() {
            Transform3D::new(Basis::IDENTITY, center)
        } else {
            let x_axis = direction.cross(Vector3::UP);
            let z_axis = x_axis.cross(direction);
            Transform3D::new(
                Basis::from_cols(x_axis, direction, z_axis).orthonormalized(),
                center,
            )
        }
    }
}

// re-export
pub use inner2d::GravityShapedCurve2D;
pub use inner3d::GravityShapedCurve3D;
