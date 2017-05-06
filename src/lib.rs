//! A uniform binary file format designed for particle physics analysis.
//!
//! When running particle physics simulations,
//! it is sometimes handy to record the data and perform analysis
//! separately from the simulation process.
//!
//! This binary format is designed to read and write particle
//! physics data to files or streams, such that the tools for analysis
//! can be reused with minimum setup.
//!
//! Particle physics data is similar to video or animation streams
//! with the difference that time is arbitrarily defined.
//! This format allows precise control over data changes,
//! by changing the offset instance id and range.
//!
//! 10 built-in Rust number types are supported,
//! using array notation, with vector and matrix dimensions up to 80x80.
//! You can also define custom binary formats.
//!
//! You can repeat the same data multiple times,
//! or write only ranges that changes.
//! Semantics is controlled by the application,
//! but the format is generic enough to reuse algorithms across projects.
//!
//!
//! ### Format
//!
//! ```ignore
//! type format == 0 => end of stream
//! type format: u16, property id: u16
//! |- bytes == 0 => no more data, next property
//! |- bytes: u64, offset instance id: u64, data: [u8; bytes]
//! ```
//!
//! Integers are stored in little-endian format.
//!
//! The number of items in the data are inferred from the number of bytes
//! and knowledge about the type format.
//!
//! ### Motivation for the design
//!
//! A file format is uniform when it organized the same way everywhere.
//!
//! One benefit with a uniform format is that you can easily split
//! data up into multiple files, or stream it across a network.
//! It is also easy to generate while running a physics simulation.
//!
//! This file format assumes that the application has some
//! form of data structure, where each particle instance
//! is assigned a unique id, and their object relations
//! can be derived from a property id.
//! Since the property ids are known, one can use external tools
//! to analyze the physical properties relative to each other,
//! without any knowledge about the data structure.
//!
//! To describe time, just create a new property id, e.g. f32 scalar,
//! and write out this value before other data in the same time step.
//!
//! ### Usage
//!
//! The traits `Scalar`, `Vector` and `Matrix` are implemented for
//! array types of primitive integer and float formats.
//!
//! When you write data to a file the order is preserved.
//!
//! ```ignore
//! use binpool::Scalar;
//!
//! let prop_id = 0; // A unique property id.
//! let data: Vec<f32> = vec![1.0, 2.0, 3.0];
//! Scalar::write_array(prop_id, &data, &mut file).unwrap();
//! ```
//!
//! When you read from a file, e.g. to replay a recorded simulation,
//! you often read a single frame at a time then wait for the time to read next frame.
//! To do this, use a loop with flags for each property and break when
//! all read flags are set.
//!
//! ```ignore
//! use binpool::State;
//!
//! let prop_id = 0; // A unique property id.
//! let mut read_prop_id = false;
//! let mut data: Vec<[f32; 2]> = vec![];
//! while let Ok((Some(state), ty, prop)) = State::read(&mut file) {
//!     match prop {
//!         prop_id if !read_prop_id => {
//!             Vector::read_array(state, ty, &mut data, &mut file).unwrap();
//!             read_prop_id = true;
//!         }
//!         _ => break,
//!     }
//! }
//! ```
//!
//! Data is often stored in a struct and overwritten for each frame.
//! The example above uses a local variable just for showing how to read data.

#![deny(missing_docs)]

use std::marker::PhantomData;
use std::io;

pub use read_write::{Array, Matrix, Vector, Scalar};

const TYPES: u16 = 10;
const SIZE: u16 = 80;

mod read_write;

/// Type format for a property.
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Type {
    /// Unsigned 8 bit integer.
    U8,
    /// Unsigned 16 bit integer.
    U16,
    /// Unsigned 32 bit integer.
    U32,
    /// Unsigned 64 bit integer.
    U64,
    /// Signed 8 bit integer.
    I8,
    /// Signed 16 bit integer.
    I16,
    /// Signed 32 bit integer.
    I32,
    /// Signed 64 bit integer.
    I64,
    /// 32 bit float.
    F32,
    /// 64 bit float.
    F64,
}

impl Type {
    /// A unique number representing each type.
    pub fn type_id(&self) -> u16 {
        match *self {
            Type::U8 => 0,
            Type::U16 => 1,
            Type::U32 => 2,
            Type::U64 => 3,
            Type::I8 => 4,
            Type::I16 => 5,
            Type::I32 => 6,
            Type::I64 => 7,
            Type::F32 => 8,
            Type::F64 => 9,
        }
    }

    /// Returns the size of type in bytes.
    pub fn type_size(&self) -> u64 {
        match *self {
            Type::U8 => 1,
            Type::U16 => 2,
            Type::U32 => 4,
            Type::U64 => 8,
            Type::I8 => 1,
            Type::I16 => 2,
            Type::I32 => 4,
            Type::I64 => 8,
            Type::F32 => 4,
            Type::F64 => 8,
        }
    }

    /// Returns the type format and size in bytes for a matrix.
    ///
    /// Notice that this method uses rows and columns, not width and height.
    ///
    /// Returns `None` if the matrix exceed dimensions 80x80.
    /// Returns `None` if the width or height is zero.
    pub fn matrix(&self, rows: u8, cols: u8) -> Option<(u16, u64)> {
        if cols == 0 || rows == 0 || cols as u16 > SIZE || rows as u16 > SIZE {
            None
        } else {
            Some((
                1 + self.type_id() * SIZE * SIZE +
                ((rows-1) as u16) * SIZE + ((cols-1) as u16),
                self.type_size() * cols as u64 * rows as u64
            ))
        }
    }

    /// Returns the type format and size in bytes for a scalar.
    pub fn scalar(&self) -> (u16, u64) {
        (1 + self.type_id() * SIZE * SIZE, self.type_size())
    }

    /// Returns the type format and size in bytes for a vector.
    ///
    /// Returns `None` if the vector exceed dimension 80.
    /// Returns `None` if the vector has dimension zero.
    pub fn vector(&self, dim: u8) -> Option<(u16, u64)> {
        if dim == 0 || dim as u16 > SIZE {
            None
        } else {
            Some((
                1 + self.type_id() * SIZE * SIZE + (dim - 1) as u16,
                self.type_size() * dim as u64
            ))
        }
    }

    /// Returns the offset for specifying a custom format.
    pub fn offset_custom_format() -> u16 {
        1 + TYPES * SIZE * SIZE
    }

    /// Returns the number of available custom formats.
    pub fn custom_formats() -> u16 {
        (((1 as u32) << 16) - Type::offset_custom_format() as u32) as u16
    }

    /// Returns the type and matrix dimensions from type format.
    pub fn info(format: u16) -> Option<(Type, u8, u8)> {
        if format == 0 || format >= Type::offset_custom_format() {
            None
        } else {
            // Remove offset at 1.
            let format = format - 1;
            let ty = format / (SIZE * SIZE);
            let rows = (format % (SIZE * SIZE)) / SIZE + 1;
            let cols = format % SIZE + 1;
            Some((match ty {
                0 => Type::U8,
                1 => Type::U16,
                2 => Type::U32,
                3 => Type::U64,
                4 => Type::I8,
                5 => Type::I16,
                6 => Type::I32,
                7 => Type::I64,
                8 => Type::F32,
                9 => Type::F64,
                _ => return None,
            }, rows as u8, cols as u8))
        }
    }
}

/// Type format state.
pub struct TypeFormat;
/// Property Id state.
pub struct PropertyId;
/// Bytes state.
pub struct Bytes;
/// Offset instance id state.
pub struct OffsetInstanceId;
/// Data state.
pub struct Data;

/// Stores the state for writing and reading.
pub struct State<T = TypeFormat>(PhantomData<T>);

impl State {
    /// Creates a new state.
    pub fn new() -> State {
        State(PhantomData)
    }

    /// Reads type format and property.
    ///
    /// Returns `None` in first argument if there is no more data.
    pub fn read<R: io::Read>(r: &mut R) -> io::Result<(Option<State<Bytes>>, u16, u16)> {
        let mut ty: u16 = 0;
        let mut property_id: u16 = 0;
        let state = State::new().read_type_format(&mut ty, r)?;
        if ty == 0 {Ok((None, 0, 0))}
        else {
            Ok((
                Some(state.read_property_id(&mut property_id, r)?),
                ty, property_id
            ))
        }
    }

    /// Writes type format.
    pub fn write_type_format<W: io::Write>(
        self,
        type_format: u16,
        w: &mut W
    ) -> io::Result<State<PropertyId>> {
        use read_write::Scalar;

        type_format.write(w)?;
        Ok(State(PhantomData))
    }

    /// Reads type format.
    pub fn read_type_format<R: io::Read>(
        self,
        type_format: &mut u16,
        r: &mut R
    ) -> io::Result<State<PropertyId>> {
        use read_write::Scalar;

        type_format.read(r)?;
        Ok(State(PhantomData))
    }

    /// Ends writing state.
    pub fn end_type_formats<W: io::Write>(self, w: &mut W) -> io::Result<()> {
        use read_write::Scalar;

        (0 as u16).write(w)?;
        Ok(())
    }
}

impl State<PropertyId> {
    /// Writes property id.
    pub fn write_property_id<W: io::Write>(
        self,
        property_id: u16,
        w: &mut W
    ) -> io::Result<State<Bytes>> {
        use read_write::Scalar;

        property_id.write(w)?;
        Ok(State(PhantomData))
    }

    /// Reads property id.
    pub fn read_property_id<R: io::Read>(
        self,
        property_id: &mut u16,
        r: &mut R
    ) -> io::Result<State<Bytes>> {
        use read_write::Scalar;

        property_id.read(r)?;
        Ok(State(PhantomData))
    }
}

impl State<Bytes> {
    /// Writes number of bytes in data.
    pub fn write_bytes<W: io::Write>(
        self,
        bytes: u64,
        w: &mut W
    ) -> io::Result<State<OffsetInstanceId>> {
        use read_write::Scalar;

        bytes.write(w)?;
        Ok(State(PhantomData))
    }

    /// Reads bytes.
    pub fn read_bytes<R: io::Read>(
        self,
        bytes: &mut u64,
        r: &mut R
    ) -> io::Result<State<OffsetInstanceId>> {
        use read_write::Scalar;

        bytes.read(r)?;
        Ok(State(PhantomData))
    }

    /// Ends byte block.
    pub fn end_bytes<W: io::Write>(
        self,
        w: &mut W
    ) -> io::Result<State<TypeFormat>> {
        use read_write::Scalar;

        (0 as u64).write(w)?;
        Ok(State(PhantomData))
    }

    /// Checks if this is the end of bytes.
    pub fn has_end_bytes<R: io::Read>(
        self,
        r: &mut R
    ) -> io::Result<State<TypeFormat>> {
        let mut val: u64 = 0;
        val.read(r)?;
        if val == 0 {
            Ok(State(PhantomData))
        } else {
            Err(io::ErrorKind::InvalidData.into())
        }
    }
}

impl State<OffsetInstanceId> {
    /// Writes offset instance id.
    pub fn write_offset_instance_id<W: io::Write>(
        self,
        offset_instance_id: u64,
        w: &mut W
    ) -> io::Result<State<Data>> {
        use read_write::Scalar;

        offset_instance_id.write(w)?;
        Ok(State(PhantomData))
    }

    /// Reads offset instance id.
    pub fn read_offset_instance_id<R: io::Read>(
        self,
        offset_instance_id: &mut u64,
        r: &mut R
    ) -> io::Result<State<Data>> {
        use read_write::Scalar;

        offset_instance_id.read(r)?;
        Ok(State(PhantomData))
    }
}

impl State<Data> {
    /// Writes data.
    pub fn write_data<W: io::Write>(
        self,
        data: &[u8],
        w: &mut W
    ) -> io::Result<State<Data>> {
        w.write(data)?;
        Ok(State(PhantomData))
    }

    /// End of data.
    pub fn end_data(self) -> State<Bytes> {
        State(PhantomData)
    }
}
