//!
//! Define a ring gravity shape
//!

use super::Shape;
use crate::gravity::{
    build_trs::TransformBuilder3D,
    util::util3d::{BASIS_Y, BASIS_Z, flatten_y},
};
use godot::{
    classes::{BoxShape3D, CapsuleShape3D, ConvexPolygonShape3D, Shape3D},
    prelude::*,
};
use std::f64::consts::{PI, TAU};

/// Define a gravity based on an axis direction.
#[derive(GodotClass)]
#[class(base=Resource)]
pub struct GravityShapeRing3D {
    base: Base<Resource>,

    /// Generated shapes
    internal: Option<Internal>,

    /// Outer radius of the ring.
    #[export(range = (0.0, 20.0, or_greater))]
    #[var(get, set = set_outer_radius)]
    outer_radius: real,

    /// Inner radius of the ring.
    #[export(range = (0.0, 20.0, or_greater))]
    #[var(get, set = set_inner_radius)]
    inner_radius: real,

    /// Height of the ring.
    #[export(range = (0.0, 10.0, or_greater))]
    #[var(get, set = set_height)]
    height: real,

    /// Radius of the edges for rounding the ring.
    #[export(range = (0.0, 1.0, or_greater))]
    #[var(get, set = set_edge_radius)]
    edge_radius: real,

    /// Number of vertices to generate
    #[export(range = (3.0, 256.0, 1.0, or_greater))]
    #[var(get, set = set_vertex_count)]
    vertex_count: u32,
}

/// Specify if we need to generate a single box shape or
/// if we need multiple one to create the rounded edges.
enum Internal {
    /// Torus or filled disc
    Torus {
        /// Central shape to fill the disc.
        fill: Option<Gd<ConvexPolygonShape3D>>,

        /// Capsule shape to roundup the edges.
        border: Ring,
    },

    /// Hollow disc
    Flat {
        /// The three shapes that makes up a slice
        slice: Slice,

        /// Transforms for the slice of shapes
        transforms: Vec<TransformBuilder3D<2, 3>>,
    },

    /// Tube of filled cylinder
    Tube {
        /// Central shape to fill the tube.
        fill: Option<Gd<ConvexPolygonShape3D>>,

        /// Capsule shape to roundup the edges.
        border: Gd<CapsuleShape3D>,

        /// Box shape to vertically fill between the two borders.
        center: Gd<BoxShape3D>,

        /// Transforms for the slice of shapes
        transforms: Vec<TransformBuilder3D<2, 2>>,

        /// Capsule shape for the vertical edges.
        vertical: Ring,
    },

    /// "Bolt" shape, similar to a tube but with extra thickness
    Bolt {
        /// The three shapes that makes up a slice
        slice: Slice,

        /// Box shape to vertically fill between the two borders.
        center: Gd<BoxShape3D>,

        /// Transforms for the slice of shapes
        transforms: Vec<TransformBuilder3D<2, 3>>,

        /// Capsule shape for the vertical edges.
        vertical: Ring,
    },
}

/// Define the three shapes that makes up a slice
struct Slice {
    /// Capsule shape to roundup the edges.
    outer: Gd<CapsuleShape3D>,

    /// Capsule shape to roundup the edges.
    inner: Gd<CapsuleShape3D>,

    /// Capsule shape to roundup the edges.
    center: Gd<BoxShape3D>,
}

macro_rules! impl_slice_shape {
    ( $shape:ident ) => {
        impl Slice {
            #[inline]
            fn $shape(&self) -> Gd<Shape3D> {
                self.$shape.clone().upcast::<Shape3D>()
            }
        }
    };
}
impl_slice_shape!(outer);
impl_slice_shape!(inner);
impl_slice_shape!(center);

/// Pair a capsule shape with a set of transforms.
struct Ring {
    /// Capsule shape to roundup the edges.
    capsule: Gd<CapsuleShape3D>,

    /// The transforms to position the capsules.
    transforms: Vec<Transform3D>,
}

impl Ring {
    /// number of elements in the ring
    #[inline]
    fn len(&self) -> usize {
        self.transforms.len()
    }

    /// add entries to the provided list
    fn add_entries(&self, list: &mut Vec<(Gd<Shape3D>, Transform3D)>) {
        let shape = self.capsule.clone().upcast::<Shape3D>();
        for transform in &self.transforms {
            list.push((shape.clone(), *transform));
        }
    }
}

impl Shape<Vector3, Shape3D, Transform3D> for GravityShapeRing3D {
    /// Pick the UP direction for a cuboid
    fn up(&self, position: &Vector3) -> Vector3 {
        let flatten = flatten_y(position);

        // compose a mask based on the position of the object relative to the ring's center
        let mut mask = {
            let dist = flatten.length();
            if dist > self.outer_radius {
                0b010 // outside of the ring
            } else if dist < self.inner_radius {
                0b001 // inside of the ring
            } else {
                0b000 // middle of the ring
            }
        };

        // check if the object is either above or below the ring
        let half = self.height * 0.5;
        if position.y.abs() > half {
            mask |= 0b100; // above the ring
        }

        // vertical sign specify if the object is above or below the ring
        let sign = position.y.sign();

        // based on the mask deduce the up direction
        match mask {
            0b001 => (-flatten).normalized_or_zero(),
            0b010 => flatten.normalized_or_zero(),
            0b100 => Vector3::new(0.0, sign, 0.0),
            0b101 => {
                let mut ref_pos = flatten.normalized() * self.inner_radius;
                ref_pos.y = sign * half;
                (*position - ref_pos).normalized_or_zero()
            }
            0b110 => {
                let mut ref_pos = flatten.normalized() * self.outer_radius;
                ref_pos.y = sign * half;
                (*position - ref_pos).normalized_or_zero()
            }

            // Both on the outside and inside of the ring,
            // this should never happen.
            _ => Vector3::new(0.0, sign, 0.0),
        }
    }

    /// Return a list of colliders
    #[inline]
    fn colliders(&mut self) -> Vec<(Gd<Shape3D>, Transform3D)> {
        // Recompute the internal shapes if requested
        if self.internal.is_none() {
            // Generate an internal collision shape representation based on current parameters
            self.internal = Some({
                let thin_border = self.inner_radius >= self.outer_radius;
                let filled = self.inner_radius <= 0.0;

                if self.height > 0.0 {
                    if thin_border || filled {
                        Internal::new_tube(
                            self.outer_radius,
                            self.height,
                            self.edge_radius,
                            self.vertex_count as usize,
                            filled,
                        )
                    } else {
                        Internal::new_bolt(
                            self.outer_radius,
                            self.inner_radius,
                            self.height,
                            self.edge_radius,
                            self.vertex_count as usize,
                        )
                    }
                } else if thin_border || filled {
                    Internal::new_torus(
                        self.outer_radius,
                        self.edge_radius,
                        self.vertex_count as usize,
                        filled,
                    )
                } else {
                    Internal::new_flat(
                        self.outer_radius,
                        self.inner_radius,
                        self.edge_radius,
                        self.vertex_count as usize,
                    )
                }
            });
        }
        self.internal.as_ref().unwrap().colliders()
    }
}

#[godot_api]
impl IResource for GravityShapeRing3D {
    fn init(base: Base<Resource>) -> Self {
        Self {
            base,
            internal: None,
            outer_radius: 15.0,
            inner_radius: 0.0,
            height: 0.0,
            edge_radius: 0.0,
            vertex_count: 24,
        }
    }
}

#[godot_api]
impl GravityShapeRing3D {
    #[func]
    fn set_outer_radius(&mut self, radius: real) {
        self.outer_radius = radius;
        // enforce that the inner radius is smaller than the outer radius
        self.inner_radius = self.inner_radius.min(self.outer_radius);
        self.internal = None;
    }

    #[func]
    fn set_inner_radius(&mut self, radius: real) {
        self.inner_radius = radius;
        // enforce that the outer radius is greater than the inner radius
        self.outer_radius = self.outer_radius.max(self.inner_radius);
        self.internal = None;
    }

    #[func]
    fn set_height(&mut self, height: real) {
        self.height = height;
        self.internal = None;
    }

    #[func]
    fn set_edge_radius(&mut self, radius: real) {
        self.edge_radius = radius;
        self.internal = None;
    }

    #[func]
    fn set_vertex_count(&mut self, count: u32) {
        self.vertex_count = count;
        self.internal = None;
    }
}

impl Internal {
    /// Create a torus shape
    fn new_torus(border_radius: real, edge_radius: real, vertex_count: usize, fill: bool) -> Self {
        // build transforms for each section of the ring
        let [(length, distance)] = compute_edges(vertex_count, &[border_radius]);
        let transforms = make_trs_ring(
            vertex_count,
            &[BASIS_Z],
            &[Vector3::new(distance, 0.0, 0.0)],
            true,
        )
        .into_iter()
        .map(Transform3D::from)
        .collect::<Vec<_>>();

        // generate a filling shape if necessary
        let fill = if fill {
            Some(make_convex(border_radius, edge_radius, vertex_count))
        } else {
            None
        };

        // Complete the torus shape
        Self::Torus {
            fill,
            border: Ring {
                capsule: make_capsule(edge_radius, length),
                transforms,
            },
        }
    }

    /// Create a hollowed disc shape
    fn new_flat(
        outer_radius: real,
        inner_radius: real,
        edge_radius: real,
        vertex_count: usize,
    ) -> Self {
        // build transforms for each section of the ring
        let width = outer_radius - inner_radius;
        let [
            (outer_length, outer_distance),
            (inner_length, inner_distance),
            (middle_length, middle_distance),
        ] = compute_edges(vertex_count, &[outer_radius, inner_radius, width * 0.5]);
        let transforms = make_trs_ring(
            vertex_count,
            &[BASIS_Z, BASIS_Y],
            &[
                Vector3::new(outer_distance, 0.0, 0.0),
                Vector3::new(inner_distance, 0.0, 0.0),
                Vector3::new(middle_distance, 0.0, 0.0),
            ],
            true,
        );

        // Complete the flatten ring shape
        Self::Flat {
            slice: Slice {
                outer: make_capsule(edge_radius, outer_length),
                inner: make_capsule(edge_radius, inner_length),
                center: make_box(width, edge_radius * 2.0, middle_length),
            },
            transforms,
        }
    }

    /// Create a tube shape
    fn new_tube(
        border_radius: real,
        height: real,
        edge_radius: real,
        vertex_count: usize,
        fill: bool,
    ) -> Self {
        // build transforms for each section of the ring
        let [(length, distance)] = compute_edges(vertex_count, &[border_radius]);
        let transforms = make_trs_ring(
            vertex_count,
            &[BASIS_Z, BASIS_Y],
            &[
                Vector3::new(distance, height * 0.5, 0.0),
                Vector3::new(distance, 0.0, 0.0),
            ],
            true,
        );

        // generate a filling shape if necessary
        let fill = if fill {
            Some(make_convex(
                border_radius,
                edge_radius + height * 0.5,
                vertex_count,
            ))
        } else {
            None
        };

        // Complete the tube shape
        Self::Tube {
            fill,
            border: make_capsule(edge_radius, length),
            center: make_box(edge_radius * 2.0, height, length),
            transforms,
            vertical: Ring {
                capsule: make_capsule(edge_radius, height),
                transforms: make_vertical_trs_ring(vertex_count, distance),
            },
        }
    }

    /// Create a bolt shape
    fn new_bolt(
        outer_radius: real,
        inner_radius: real,
        height: real,
        edge_radius: real,
        vertex_count: usize,
    ) -> Self {
        // build transforms for each section of the ring
        let width = outer_radius - inner_radius;
        let [
            (outer_length, outer_distance),
            (inner_length, inner_distance),
            (middle_length, middle_distance),
        ] = compute_edges(vertex_count, &[outer_radius, inner_radius, width * 0.5]);
        let transforms = {
            let half_height = height * 0.5;
            make_trs_ring(
                vertex_count,
                &[BASIS_Z, BASIS_Y],
                &[
                    Vector3::new(outer_distance, half_height, 0.0),
                    Vector3::new(inner_distance, half_height, 0.0),
                    Vector3::new(middle_distance, 0.0, 0.0),
                ],
                true,
            )
        };

        // Complete the bolt shape
        Self::Bolt {
            slice: Slice {
                outer: make_capsule(edge_radius, outer_length),
                inner: make_capsule(edge_radius, inner_length),
                center: make_box(width, height, middle_length),
            },
            center: make_box(width, height, middle_length),
            transforms,
            vertical: Ring {
                capsule: make_capsule(edge_radius, height),
                transforms: make_vertical_trs_ring(vertex_count, outer_distance),
            },
        }
    }

    /// Return a list of colliders
    fn colliders(&self) -> Vec<(Gd<Shape3D>, Transform3D)> {
        macro_rules! negate {
            ( $x:expr ) => {
                $x = -$x;
            };
        }

        match self {
            Self::Torus { fill, border } => {
                let mut colliders = Vec::with_capacity(border.len() + 1);
                border.add_entries(&mut colliders);
                if let Some(fill) = fill {
                    colliders.push((fill.clone().upcast::<Shape3D>(), Transform3D::IDENTITY));
                }
                colliders
            }
            Self::Flat { slice, transforms } => {
                let outer = slice.outer();
                let inner = slice.inner();
                let center = slice.center();
                let mut colliders = Vec::with_capacity(transforms.len() * 3);
                for transform in transforms {
                    colliders.push((outer.clone(), transform.build(0, 0)));
                    colliders.push((inner.clone(), transform.build(0, 1)));
                    colliders.push((center.clone(), transform.build(1, 2)));
                }
                colliders
            }
            Self::Tube {
                fill,
                border,
                center,
                transforms,
                vertical,
            } => {
                // prepare the shapes
                let border = border.clone().upcast::<Shape3D>();
                let center = center.clone().upcast::<Shape3D>();

                // allocate enough room for all colliders
                let mut colliders = Vec::with_capacity(transforms.len() * 3 + vertical.len() + 1);
                for transform in transforms {
                    // position the two capsule borders
                    let mut trs = transform.build(0, 0);
                    colliders.push((border.clone(), trs));
                    negate!(trs.origin.y);
                    colliders.push((border.clone(), trs));

                    // position the wall in the center
                    colliders.push((center.clone(), transform.build(1, 1)));
                }
                // add the vertical capsules
                vertical.add_entries(&mut colliders);
                if let Some(fill) = fill {
                    colliders.push((fill.clone().upcast::<Shape3D>(), Transform3D::IDENTITY));
                }
                colliders
            }
            Self::Bolt {
                slice,
                center,
                transforms,
                vertical,
            } => {
                let outer = slice.outer();
                let inner = slice.inner();
                let center1 = slice.center();
                let center2 = center.clone().upcast::<Shape3D>();

                // allocate enough room for all colliders
                let mut colliders = Vec::with_capacity(transforms.len() * 6 + vertical.len() + 1);
                for transform in transforms {
                    // position the border capsules
                    let mut outer_trs = transform.build(0, 0);
                    let mut inner_trs = transform.build(0, 0);
                    colliders.push((outer.clone(), outer_trs));
                    colliders.push((inner.clone(), inner_trs));

                    negate!(outer_trs.origin.y);
                    negate!(inner_trs.origin.y);
                    colliders.push((outer.clone(), outer_trs));
                    colliders.push((inner.clone(), inner_trs));

                    // position the inner walls
                    let trs = transform.build(1, 1);
                    colliders.push((center1.clone(), trs));
                    colliders.push((center2.clone(), trs));
                }
                // add the vertical capsules
                vertical.add_entries(&mut colliders);
                colliders
            }
        }
    }
}

/// Build a set of transforms
fn make_trs_ring<const B: usize, const O: usize>(
    vertex_count: usize,
    initials: &[Basis; B],
    offsets: &[Vector3; O],
    shift: bool,
) -> Vec<TransformBuilder3D<B, O>> {
    let angle = (TAU as real) / vertex_count as real;
    let ang_start = if shift { angle * 0.5 } else { 0.0 };

    // build a set of transforms
    let mut transforms = Vec::with_capacity(vertex_count);
    for i in 0..vertex_count {
        let angle = ang_start + angle * i as real;

        // build basis
        let mut basis = [Basis::IDENTITY; B];
        for j in 0..B {
            basis[j] = initials[j].rotated(Vector3::UP, angle);
        }

        // build origin
        let mut origin = [Vector3::ZERO; O];
        for j in 0..O {
            origin[j] = offsets[j].rotated(Vector3::UP, angle);
        }

        transforms.push(TransformBuilder3D::new(basis, origin));
    }
    transforms
}

/// Build a set of transform for positionning vertical capsules
fn make_vertical_trs_ring(vertex_count: usize, distance: real) -> Vec<Transform3D> {
    make_trs_ring(
        vertex_count,
        &[BASIS_Y],
        &[Vector3::new(distance, 0.0, 0.0)],
        false,
    )
    .into_iter()
    .map(Transform3D::from)
    .collect::<Vec<_>>()
}

/// Find the length of an edge and its distance from the center
fn compute_edges<const N: usize>(vertex_count: usize, radiuses: &[real; N]) -> [(real, real); N] {
    let ang = (PI as real) / vertex_count as real;

    // length of the edges and distance from the center
    let mut edges = [(0.0, 0.0); N];
    for i in 0..N {
        let radius = radiuses[i];
        let length = 2.0 * radius * ang.sin();
        let distance = radius * ang.cos();
        edges[i] = (length, distance);
    }
    edges
}

/// Create a box shape
#[inline]
fn make_box(x: real, y: real, z: real) -> Gd<BoxShape3D> {
    let mut abox = BoxShape3D::new_gd();
    abox.set_size(Vector3::new(x, y, z));
    abox
}

/// Create a capsule shape
#[inline]
fn make_capsule(radius: real, height: real) -> Gd<CapsuleShape3D> {
    let mut capsule = CapsuleShape3D::new_gd();
    capsule.set_radius(radius);
    capsule.set_height(height + radius * 2.0);
    capsule
}

/// Create a convex shape for filling the ring
fn make_convex(radius: real, half_height: real, vertex_count: usize) -> Gd<ConvexPolygonShape3D> {
    let ang = (TAU as real) / vertex_count as real;

    // build the vertices
    let mut points = PackedVector3Array::new();
    points.resize(vertex_count * 2);
    for i in 0..vertex_count {
        // find where to place the point along the circle
        let pt = Vector2::new(radius, 0.0).rotated(ang * i as real);
        let j = i * 2;
        points[j] = Vector3::new(pt.x, half_height, pt.y);
        points[j + 1] = Vector3::new(pt.x, -half_height, pt.y);
    }

    // create the convex shape
    let mut convex = ConvexPolygonShape3D::new_gd();
    convex.set_points(&points);
    convex
}
