//! A cell grid to draw into directly, for the drawing a component cannot do for you.
//!
//! This is the escape hatch: a rectangle, and the cells inside it, at whichever of the three
//! resolutions the terminal offers — one mark a cell, four quadrants a cell, or eight braille dots
//! a cell. A diagram, a game board, a mandelbrot: things with no component because they have no
//! shape in common.
//!
//! What a canvas does not do is manage anything. There is no retained picture, no dirty tracking of
//! its own and no coordinate system but the one you choose; it is the drawing verbs with a
//! resolution ladder over them.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &[];
