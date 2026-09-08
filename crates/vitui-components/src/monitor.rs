//! Gauges and meters over a live value.
//!
//! The system family: the shapes an application draws when it is reporting on something that keeps
//! changing — load, throughput, temperature, queue depth. There is no v1 component of its own here,
//! because every one of those screens is a [`crate::indicate::meter`], a
//! [`crate::chart::plot`] or a [`crate::collect::table`] over data the application already has.
//!
//! The module exists so that the family has a home when one of those screens turns out to need a
//! mechanism the three above cannot express.

/// The components homed in this module. See [`crate::Family::members`].
pub const MEMBERS: &[&str] = &[];
