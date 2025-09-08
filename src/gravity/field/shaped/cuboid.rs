//!
//! Define a cuboid resource for shape backed gravity fields
//!

/// Simple macro to prepare 2D and 3D cuboid shapes
macro_rules! shape_cuboid {
    (
        $shape_type:ident where {
            $transform:ty,
            $vector:ty,
            $shape:ty
        }
    ) => {
        // alias provided types
        type Transform = $transform;
        type Vector = $vector;
        type GShape = $shape;

        /// Define a gravity based on an axis direction.
        #[derive(GodotClass)]
        #[class(base=Resource)]
        pub struct $shape_type {
            base: Base<Resource>,

            /// Generated shapes
            internal: Option<Internal>,

            /// Define the size of the cuboid.
            #[export]
            #[var(get, set = set_box_size)]
            box_size: Vector,

            /// Radius of the edges for rounding the box.
            #[export(range = (0.0, 1.0, or_greater))]
            #[var(get, set = set_edge_radius)]
            edge_radius: real,

            /// Is the cuboid filled or hollow?
            /// This has no effect if the edge radius is zero.
            #[export]
            #[var(get, set = set_hollow)]
            hollow: bool,
        }

        #[godot_api]
        impl IResource for $shape_type {
            fn init(base: Base<Resource>) -> Self {
                Self {
                    base,
                    internal: None,
                    box_size: Vector::ONE,
                    edge_radius: 0.0,
                    hollow: false,
                }
            }
        }

        #[godot_api]
        impl $shape_type {
            #[func]
            fn set_box_size(&mut self, size: Vector) {
                self.box_size = size.coord_max(MIN_SIZE);
                self.internal = None;
            }

            #[func]
            fn set_edge_radius(&mut self, radius: real) {
                self.edge_radius = radius;
                self.internal = None;
            }

            #[func]
            fn set_hollow(&mut self, hollow: bool) {
                self.hollow = hollow;
                self.internal = None;
            }
        }

        impl Shape<Vector, GShape, Transform> for $shape_type {
            /// Pick the UP direction for a cuboid
            #[inline]
            fn up(&self, position: &Vector) -> Vector {
                self.up_func(position)
            }

            /// Return a list of colliders
            fn colliders(&mut self) -> Vec<(Gd<GShape>, Transform)> {
                // Recompute the internal shapes if requested
                if self.internal.is_none() {
                    self.internal =
                        Some(Internal::new(&self.box_size, self.edge_radius, self.hollow));
                }

                // Ask the internal shape for its colliders set
                self.internal.as_ref().unwrap().colliders()
            }
        }
    };
}

pub mod inner2d {

    use crate::{
        gravity::{build_trs::TransformBuilder2D, field::shaped::Shape, util::util2d::*},
        unit,
    };
    use godot::{
        classes::{CapsuleShape2D, RectangleShape2D, Shape2D},
        prelude::*,
    };

    shape_cuboid! {
        GravityShapedCuboid2D where {
            Transform2D,
            Vector2,
            Shape2D
        }
    }

    impl GravityShapedCuboid2D {
        /// Pick the UP direction for a cuboid
        fn up_func(&self, position: &Vector2) -> Vector2 {
            // use a bitmask to deduce the strategy to use
            let mut mask = 0b00;
            macro_rules! set {
                ( $coord:ident => $bit:literal ) => {
                    if position.$coord.abs() > self.box_size.$coord {
                        mask |= $bit;
                    }
                };
            }
            set![ x => 0b01 ];
            set![ y => 0b10 ];

            match mask {
                // over one of the six faces
                0b01 => Vector2::new(position.x.sign(), 0.0),
                0b10 => Vector2::new(0.0, position.y.sign()),

                // over one of the eight corners
                0b11 => (self.box_size * position.sign()).direction_to(*position),

                // Inside of the box, should not happen
                _ => position.normalized_or_zero(),
            }
        }
    }

    /// Specify if we need to generate a single box shape or
    /// if we need multiple one to create the rounded edges.
    struct Internal {
        /// One box for each of the three side of the box.
        /// Should be None if the cuboid is hollow.
        center: Option<Gd<RectangleShape2D>>,

        /// Capsules for each set of four parallel edges.
        edges: Box<[(Gd<CapsuleShape2D>, TransformBuilder2D<1, 2>); 2]>,
    }

    impl Internal {
        /// Create a rounded box shape
        fn new(size: &Vector2, radius: real, hollow: bool) -> Self {
            // Create a shape
            let diameter = radius * 2.0;
            macro_rules! edge {
                ( $coord:ident ) => {{
                    let mut edge = CapsuleShape2D::new_gd();
                    edge.set_radius(radius);
                    edge.set_height(size.$coord + diameter);
                    edge
                }};
            }
            macro_rules! pos {
                ( $x:tt , $y:tt ) => {
                    #[allow(clippy::neg_multiply)]
                    Vector2::new(size.x * unit![$x], size.y * unit![$y])
                };
            }

            // create the faces
            let center = if hollow {
                None
            } else {
                let mut center = RectangleShape2D::new_gd();

                center.set_size(*size);
                // Create boxes for the six faces
                Some(center)
            };

            // prepare the transforms for the twelve edges
            let edges = Box::new([
                (
                    edge![x],
                    TransformBuilder2D::new([ROT_X], [pos![ 0, + ], pos![ 0, - ]]),
                ),
                (
                    edge![y],
                    TransformBuilder2D::new([ROT_Y], [pos![ +, 0 ], pos![ -, 0 ]]),
                ),
            ]);

            // create the rounded shape
            Self { center, edges }
        }

        /// Return a list of colliders
        fn colliders(&self) -> Vec<(Gd<Shape2D>, Transform2D)> {
            macro_rules! cast_shape {
                ( $shape:expr, $trs:expr ) => {
                    (($shape).clone().upcast::<Shape2D>(), $trs)
                };
            }

            // allocate a vector to store the shapes
            let size = if self.center.is_some() { 5 } else { 4 };
            let mut shapes = Vec::with_capacity(size);

            // Push the internal boxes into the list
            if let Some(center) = &self.center {
                shapes.push(cast_shape![center, Transform2D::IDENTITY]);
            }

            // add the shapes for the edges
            for (edge, trs) in self.edges.iter() {
                for i in 0..4 {
                    shapes.push(cast_shape![edge, trs.build(0, i)]);
                }
            }
            shapes
        }
    }
}

pub mod inner3d {

    use crate::{
        gravity::{build_trs::TransformBuilder3D, field::shaped::Shape, util::util3d::*},
        unit,
    };
    use godot::{
        classes::{BoxShape3D, CapsuleShape3D, Shape3D},
        prelude::*,
    };

    shape_cuboid! {
        GravityShapedCuboid3D where {
            Transform3D,
            Vector3,
            Shape3D
        }
    }

    impl GravityShapedCuboid3D {
        /// Pick the UP direction for a cuboid
        fn up_func(&self, position: &Vector3) -> Vector3 {
            // use a bitmask to deduce the strategy to use
            let mut mask = 0b000;
            macro_rules! set {
                ( $coord:ident => $bit:literal ) => {
                    if position.$coord.abs() > self.box_size.$coord {
                        mask |= $bit;
                    }
                };
            }
            set![ x => 0b001 ];
            set![ y => 0b010 ];
            set![ z => 0b100 ];

            // Flatten the vector along a axis aligned plane
            macro_rules! flatten {
                ( $func:ident ) => {{
                    let flat = $func(position);
                    (self.box_size * flat.sign()).direction_to(flat)
                }};
            }
            match mask {
                // over one of the six faces
                0b001 => Vector3::new(position.x.sign(), 0.0, 0.0),
                0b010 => Vector3::new(0.0, position.y.sign(), 0.0),
                0b100 => Vector3::new(0.0, 0.0, position.z.sign()),

                // over one of the twelve edges
                0b011 => flatten![flatten_z],
                0b101 => flatten![flatten_y],
                0b110 => flatten![flatten_x],

                // over one of the eight corners
                0b111 => (self.box_size * position.sign()).direction_to(*position),

                // Inside of the box, should not happen
                _ => position.normalized_or_zero(),
            }
        }
    }

    /// Specify if we need to generate a single box shape or
    /// if we need multiple one to create the rounded edges.
    struct Internal {
        /// One box for each of the three side of the box.
        /// Should be None if the cuboid is hollow.
        faces: Option<[Gd<BoxShape3D>; 3]>,

        /// Capsules for each set of four parallel edges.
        edges: Box<[(Gd<CapsuleShape3D>, TransformBuilder3D<1, 4>); 3]>,
    }

    impl Internal {
        /// Create a rounded box shape
        fn new(size: &Vector3, radius: real, hollow: bool) -> Self {
            // Create a shape
            let diameter = radius * 2.0;
            macro_rules! face {
                ( $coord:ident ) => {{
                    let mut face = BoxShape3D::new_gd();
                    let mut size = *size;
                    size.$coord += diameter;
                    face.set_size(size);
                    face
                }};
            }
            macro_rules! edge {
                ( $coord:ident ) => {{
                    let mut edge = CapsuleShape3D::new_gd();
                    edge.set_radius(radius);
                    edge.set_height(size.$coord + diameter);
                    edge
                }};
            }
            macro_rules! pos {
                (  $x:tt , $y:tt , $z:tt ) => {
                    #[allow(clippy::neg_multiply)]
                    Vector3::new(size.x * unit![$x], size.y * unit![$y], size.z * unit![$z])
                };
            }

            // create the faces
            let faces = if hollow {
                None
            } else {
                // Create boxes for the six faces
                Some([face![x], face![y], face![z]])
            };

            // prepare the transforms for the twelve edges
            let edges = Box::new([
                (
                    edge![x],
                    TransformBuilder3D::new(
                        [BASIS_X],
                        [
                            pos![ 0, +, + ],
                            pos![ 0, +, - ],
                            pos![ 0, -, - ],
                            pos![ 0, -, + ],
                        ],
                    ),
                ),
                (
                    edge![y],
                    TransformBuilder3D::new(
                        [BASIS_Y],
                        [
                            pos![ +, 0, + ],
                            pos![ -, 0, + ],
                            pos![ -, 0, - ],
                            pos![ +, 0, - ],
                        ],
                    ),
                ),
                (
                    edge![z],
                    TransformBuilder3D::new(
                        [BASIS_Z],
                        [
                            pos![ +, +, 0 ],
                            pos![ -, +, 0 ],
                            pos![ -, -, 0 ],
                            pos![ +, -, 0 ],
                        ],
                    ),
                ),
            ]);

            // create the rounded shape
            Self { faces, edges }
        }

        /// Return a list of colliders
        fn colliders(&self) -> Vec<(Gd<Shape3D>, Transform3D)> {
            macro_rules! cast_shape {
                ( $shape:expr, $trs:expr ) => {
                    (($shape).clone().upcast::<Shape3D>(), $trs)
                };
            }

            // allocate a vector to store the shapes
            let size = if self.faces.is_some() { 3 * 5 } else { 3 * 4 };
            let mut shapes = Vec::with_capacity(size);

            // Push the internal boxes into the list
            if let Some(faces) = &self.faces {
                for face in faces {
                    shapes.push(cast_shape![face, Transform3D::IDENTITY]);
                }
            }

            // add the shapes for the edges
            for (edge, trs) in self.edges.iter() {
                for i in 0..4 {
                    shapes.push(cast_shape![edge, trs.build(0, i)]);
                }
            }
            shapes
        }
    }
}

// re-export
pub use inner2d::GravityShapedCuboid2D;
pub use inner3d::GravityShapedCuboid3D;
