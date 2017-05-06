use std::io;

use Bytes;
use State;
use Type;

/// Implemented by array types.
pub trait Array {
    /// The type of item.
    type Item;

    /// Returns the number of items.
    fn len(&self) -> usize;
    /// Get value of item by index.
    fn get(&self, ind: usize) -> &Self::Item;
    /// Set value of item at index.
    fn set(&mut self, ind: usize, val: Self::Item);
    /// Push new item at the end of array.
    fn push(&mut self, val: Self::Item);
}

impl<T> Array for Vec<T> {
    type Item = T;

    fn len(&self) -> usize {self.len()}
    fn get(&self, ind: usize) -> &T {&self[ind]}
    fn set(&mut self, ind: usize, val: T) {self[ind] = val}
    fn push(&mut self, val: T) {
        Vec::push(self, val)
    }
}

/// Implemented by matrix types.
pub trait Matrix: Sized + Default {
    /// Scalar type.
    type Scalar: Scalar;

    /// Returns dimensions.
    fn dim() -> [usize; 2];
    /// Gets value.
    fn get(&self, row: usize, col: usize) -> &Self::Scalar;
    /// Sets value.
    fn set(&mut self, row: usize, col: usize, val: Self::Scalar);

    /// Writes property.
    fn write_property<W: io::Write>(&self, property_id: u16, w: &mut W) -> io::Result<()> {
        let dim = <Self as Matrix>::dim();
        let (ty, s) = <Self::Scalar as Scalar>::ty().matrix(dim[0] as u8, dim[1] as u8).unwrap();
        let state = State::new()
            .write_type_format(ty, w)?
            .write_property_id(property_id, w)?
            .write_bytes(s, w)?
            .write_offset_instance_id(0, w)?;
        for i in 0..dim[0] {
            for j in 0..dim[1] {
                self.get(i, j).write(w)?;
            }
        }
        state.end_data().end_bytes(w)?;
        Ok(())
    }

    /// Writes array.
    fn write_array<W: io::Write, A: Array<Item = Self>>(
        property_id: u16,
        arr: &A,
        w: &mut W
    ) -> io::Result<()> {
        let dim = <Self as Matrix>::dim();
        let n = arr.len();
        let (ty, s) = <Self::Scalar as Scalar>::ty().matrix(dim[0] as u8, dim[1] as u8).unwrap();
        let state = State::new()
            .write_type_format(ty, w)?
            .write_property_id(property_id, w)?
            .write_bytes(s * n as u64, w)?
            .write_offset_instance_id(0, w)?;
        for k in 0..n {
            let mat = arr.get(k);
            for i in 0..dim[0] {
                for j in 0..dim[1] {
                    mat.get(i, j).write(w)?;
                }
            }
        }
        state.end_data().end_bytes(w)?;
        Ok(())
    }

    /// Reads property.
    fn read_property<R: io::Read>(
        &mut self,
        state: State<Bytes>,
        ty: u16,
        r: &mut R
    ) -> io::Result<()> {
        let dim = Self::dim();
        let self_ty = Self::Scalar::ty();
        if let Some((ty, rows, cols)) = Type::info(ty) {
            if ty == self_ty && rows == dim[0] as u8 && cols == dim[1] as u8 {
                let mut bytes = 0;
                let state = state.read_bytes(&mut bytes, r)?;
                let (_, scalar_bytes) = self_ty.matrix(dim[0] as u8, dim[1] as u8).unwrap();
                if bytes == scalar_bytes {
                    let mut offset = 0;
                    let state = state.read_offset_instance_id(&mut offset, r)?;
                    if offset == 0 {
                        for i in 0..dim[0] {
                            for j in 0..dim[1] {
                                let mut scalar: Self::Scalar = Default::default();
                                scalar.read(r)?;
                                self.set(i, j, scalar);
                            }
                        }
                        state.end_data().has_end_bytes(r)?;
                        return Ok(())
                    }
                }
            }
        }
        return Err(io::ErrorKind::InvalidData.into())
    }

    /// Reads array.
    fn read_array<R: io::Read, A: Array<Item = Self>>(
        state: State<Bytes>,
        ty: u16,
        arr: &mut A,
        r: &mut R
    ) -> io::Result<()> {
        use std::usize;

        let dim = <Self as Matrix>::dim();
        let self_ty = <Self::Scalar as Scalar>::ty();
        if let Some((ty, rows, cols)) = Type::info(ty) {
            if ty == self_ty && rows == dim[0] as u8 && cols == dim[1] as u8 {
                let mut bytes = 0;
                let state = state.read_bytes(&mut bytes, r)?;
                let (_, scalar_bytes) = self_ty.matrix(dim[0] as u8, dim[1] as u8).unwrap();
                if bytes % scalar_bytes == 0 {
                    let n = bytes / scalar_bytes;
                    let mut offset = 0;
                    let state = state.read_offset_instance_id(&mut offset, r)?;
                    for i in offset..(offset + n) {
                        if i > usize::MAX as u64 {
                            return Err(io::ErrorKind::Other.into());
                        }
                        while i as usize >= arr.len() {
                            arr.push(Default::default());
                        }
                        let mut vector: Self = Default::default();
                        for i in 0..dim[0] {
                            for j in 0..dim[1] {
                                let mut scalar: Self::Scalar = Default::default();
                                scalar.read(r)?;
                                vector.set(i, j, scalar);
                            }
                        }
                        arr.set(i as usize, vector);
                    }
                    state.end_data().has_end_bytes(r)?;
                    return Ok(())
                }
            }
        }
        return Err(io::ErrorKind::InvalidData.into())
    }
}

impl<T: Scalar> Matrix for [[T; 2]; 2] {
    type Scalar = T;

    #[inline]
    fn dim() -> [usize; 2] {[2, 2]}
    #[inline]
    fn get(&self, row: usize, col: usize) -> &T {&self[row][col]}
    #[inline]
    fn set(&mut self, row: usize, col: usize, val: T) {self[row][col] = val}
}

impl<T: Scalar> Matrix for [[T; 2]; 3] {
    type Scalar = T;

    #[inline]
    fn dim() -> [usize; 2] {[3, 2]}
    #[inline]
    fn get(&self, row: usize, col: usize) -> &T {&self[row][col]}
    #[inline]
    fn set(&mut self, row: usize, col: usize, val: T) {self[row][col] = val}
}

impl<T: Scalar> Matrix for [[T; 2]; 4] {
    type Scalar = T;

    #[inline]
    fn dim() -> [usize; 2] {[4, 2]}
    #[inline]
    fn get(&self, row: usize, col: usize) -> &T {&self[row][col]}
    #[inline]
    fn set(&mut self, row: usize, col: usize, val: T) {self[row][col] = val}
}

impl<T: Scalar> Matrix for [[T; 3]; 2] {
    type Scalar = T;

    #[inline]
    fn dim() -> [usize; 2] {[2, 3]}
    #[inline]
    fn get(&self, row: usize, col: usize) -> &T {&self[row][col]}
    #[inline]
    fn set(&mut self, row: usize, col: usize, val: T) {self[row][col] = val}
}


impl<T: Scalar> Matrix for [[T; 3]; 3] {
    type Scalar = T;

    #[inline]
    fn dim() -> [usize; 2] {[3, 3]}
    #[inline]
    fn get(&self, row: usize, col: usize) -> &T {&self[row][col]}
    #[inline]
    fn set(&mut self, row: usize, col: usize, val: T) {self[row][col] = val}
}

impl<T: Scalar> Matrix for [[T; 3]; 4] {
    type Scalar = T;

    #[inline]
    fn dim() -> [usize; 2] {[4, 3]}
    #[inline]
    fn get(&self, row: usize, col: usize) -> &T {&self[row][col]}
    #[inline]
    fn set(&mut self, row: usize, col: usize, val: T) {self[row][col] = val}
}

impl<T: Scalar> Matrix for [[T; 4]; 2] {
    type Scalar = T;

    #[inline]
    fn dim() -> [usize; 2] {[2, 4]}
    #[inline]
    fn get(&self, row: usize, col: usize) -> &T {&self[row][col]}
    #[inline]
    fn set(&mut self, row: usize, col: usize, val: T) {self[row][col] = val}
}

impl<T: Scalar> Matrix for [[T; 4]; 3] {
    type Scalar = T;

    #[inline]
    fn dim() -> [usize; 2] {[3, 4]}
    #[inline]
    fn get(&self, row: usize, col: usize) -> &T {&self[row][col]}
    #[inline]
    fn set(&mut self, row: usize, col: usize, val: T) {self[row][col] = val}
}

impl<T: Scalar> Matrix for [[T; 4]; 4] {
    type Scalar = T;

    #[inline]
    fn dim() -> [usize; 2] {[4, 4]}
    #[inline]
    fn get(&self, row: usize, col: usize) -> &T {&self[row][col]}
    #[inline]
    fn set(&mut self, row: usize, col: usize, val: T) {self[row][col] = val}
}

/// Implemented by vector types.
pub trait Vector: Sized + Default {
    /// Scalar type.
    type Scalar: Scalar;

    /// Returns the number of dimensions.
    fn dim() -> usize;
    /// Gets value.
    fn get(&self, ind: usize) -> &Self::Scalar;
    /// Sets value.
    fn set(&mut self, ind: usize, val: Self::Scalar);

    /// Writes property.
    fn write_property<W: io::Write>(&self, property_id: u16, w: &mut W) -> io::Result<()> {
        let dim = <Self as Vector>::dim();
        let (ty, s) = <Self::Scalar as Scalar>::ty().vector(dim as u8).unwrap();
        let state = State::new()
            .write_type_format(ty, w)?
            .write_property_id(property_id, w)?
            .write_bytes(s, w)?
            .write_offset_instance_id(0, w)?;
        for i in 0..dim {
            self.get(i).write(w)?;
        }
        state.end_data().end_bytes(w)?;
        Ok(())
    }

    /// Writes array.
    fn write_array<W: io::Write, A: Array<Item = Self>>(
        property_id: u16,
        arr: &A,
        w: &mut W
    ) -> io::Result<()> {
        let dim = <Self as Vector>::dim();
        let n = arr.len();
        let (ty, s) = <Self::Scalar as Scalar>::ty().vector(dim as u8).unwrap();
        let state = State::new()
            .write_type_format(ty, w)?
            .write_property_id(property_id, w)?
            .write_bytes(s * n as u64, w)?
            .write_offset_instance_id(0, w)?;
        for k in 0..n {
            let v = arr.get(k);
            for i in 0..dim {
                v.get(i).write(w)?;
            }
        }
        state.end_data().end_bytes(w)?;
        Ok(())
    }

    /// Reads property.
    fn read_property<R: io::Read>(
        &mut self,
        state: State<Bytes>,
        ty: u16,
        r: &mut R
    ) -> io::Result<()> {
        let dim = Self::dim();
        let self_ty = Self::Scalar::ty();
        if let Some((ty, rows, cols)) = Type::info(ty) {
            if ty == self_ty && rows == 1 && cols == dim as u8 {
                let mut bytes = 0;
                let state = state.read_bytes(&mut bytes, r)?;
                let (_, scalar_bytes) = self_ty.vector(dim as u8).unwrap();
                if bytes == scalar_bytes {
                    let mut offset = 0;
                    let state = state.read_offset_instance_id(&mut offset, r)?;
                    if offset == 0 {
                        for i in 0..dim {
                            let mut scalar: Self::Scalar = Default::default();
                            scalar.read(r)?;
                            self.set(i, scalar);
                        }
                        state.end_data().has_end_bytes(r)?;
                        return Ok(())
                    }
                }
            }
        }
        return Err(io::ErrorKind::InvalidData.into())
    }

    /// Reads array.
    fn read_array<R: io::Read, A: Array<Item = Self>>(
        state: State<Bytes>,
        ty: u16,
        arr: &mut A,
        r: &mut R
    ) -> io::Result<()> {
        use std::usize;

        let dim = <Self as Vector>::dim();
        let self_ty = <Self::Scalar as Scalar>::ty();
        if let Some((ty, rows, cols)) = Type::info(ty) {
            if ty == self_ty && rows == 1 && cols == dim as u8 {
                let mut bytes = 0;
                let state = state.read_bytes(&mut bytes, r)?;
                let (_, scalar_bytes) = self_ty.vector(dim as u8).unwrap();
                if bytes % scalar_bytes == 0 {
                    let n = bytes / scalar_bytes;
                    let mut offset = 0;
                    let state = state.read_offset_instance_id(&mut offset, r)?;
                    for i in offset..(offset + n) {
                        if i > usize::MAX as u64 {
                            return Err(io::ErrorKind::Other.into());
                        }
                        while i as usize >= arr.len() {
                            arr.push(Default::default());
                        }
                        let mut vector: Self = Default::default();
                        for i in 0..dim {
                            let mut scalar: Self::Scalar = Default::default();
                            scalar.read(r)?;
                            vector.set(i, scalar);
                        }
                        arr.set(i as usize, vector);
                    }
                    state.end_data().has_end_bytes(r)?;
                    return Ok(())
                }
            }
        }
        return Err(io::ErrorKind::InvalidData.into())
    }
}

impl<T: Scalar> Vector for [T; 2] {
    type Scalar = T;

    #[inline]
    fn dim() -> usize {2}
    #[inline]
    fn get(&self, ind: usize) -> &T {&self[ind]}
    #[inline]
    fn set(&mut self, ind: usize, val: T) {self[ind] = val}
}

impl<T: Scalar> Vector for [T; 3] {
    type Scalar = T;

    #[inline]
    fn dim() -> usize {3}
    #[inline]
    fn get(&self, ind: usize) -> &T {&self[ind]}
    #[inline]
    fn set(&mut self, ind: usize, val: T) {self[ind] = val}
}

impl<T: Scalar> Vector for [T; 4] {
    type Scalar = T;

    #[inline]
    fn dim() -> usize {4}
    #[inline]
    fn get(&self, ind: usize) -> &T {&self[ind]}
    #[inline]
    fn set(&mut self, ind: usize, val: T) {self[ind] = val}
}

/// Implemented by scalar values.
pub trait Scalar: Sized + Default {
    /// Type of scalar.
    fn ty() -> Type;
    /// Write to binary.
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize>;
    /// Read from binary.
    fn read<R: io::Read>(&mut self, r: &mut R) -> io::Result<usize>;

    /// Writes property.
    fn write_property<W: io::Write>(&self, property_id: u16, w: &mut W) -> io::Result<()> {
        let (ty, s) = <Self as Scalar>::ty().scalar();
        let state = State::new()
            .write_type_format(ty, w)?
            .write_property_id(property_id, w)?
            .write_bytes(s, w)?
            .write_offset_instance_id(0, w)?;
        self.write(w)?;
        state.end_data().end_bytes(w)?;
        Ok(())
    }

    /// Writes array.
    fn write_array<W: io::Write, A: Array<Item = Self>>(
        property_id: u16,
        arr: &A,
        w: &mut W
    ) -> io::Result<()> {
        let n = arr.len();
        let (ty, s) = <Self as Scalar>::ty().scalar();
        let state = State::new()
            .write_type_format(ty, w)?
            .write_property_id(property_id, w)?
            .write_bytes(s * n as u64, w)?
            .write_offset_instance_id(0, w)?;
        for k in 0..n {
            let s = arr.get(k);
            s.write(w)?;
        }
        state.end_data().end_bytes(w)?;
        Ok(())
    }

    /// Reads property.
    fn read_property<R: io::Read>(&mut self, state: State<Bytes>, ty: u16, r: &mut R) -> io::Result<()> {
        let self_ty = <Self as Scalar>::ty();
        if let Some((ty, rows, cols)) = Type::info(ty) {
            if ty == self_ty && rows == 1 && cols == 1 {
                let mut bytes = 0;
                let state = state.read_bytes(&mut bytes, r)?;
                let (_, scalar_bytes) = self_ty.scalar();
                if bytes == scalar_bytes {
                    let mut offset = 0;
                    let state = state.read_offset_instance_id(&mut offset, r)?;
                    if offset == 0 {
                        self.read(r)?;
                        state.end_data().has_end_bytes(r)?;
                        return Ok(())
                    }
                }
            }
        }
        return Err(io::ErrorKind::InvalidData.into())
    }

    /// Reads array.
    fn read_array<R: io::Read, A: Array<Item = Self>>(
        state: State<Bytes>,
        ty: u16,
        arr: &mut A,
        r: &mut R
    ) -> io::Result<()> {
        use std::usize;

        let self_ty = <Self as Scalar>::ty();
        if let Some((ty, rows, cols)) = Type::info(ty) {
            if ty == self_ty && rows == 1 && cols == 1 {
                let mut bytes = 0;
                let state = state.read_bytes(&mut bytes, r)?;
                let (_, scalar_bytes) = self_ty.scalar();
                if bytes % scalar_bytes == 0 {
                    let n = bytes / scalar_bytes;
                    let mut offset = 0;
                    let state = state.read_offset_instance_id(&mut offset, r)?;
                    for i in offset..(offset + n) {
                        if i > usize::MAX as u64 {
                            return Err(io::ErrorKind::Other.into());
                        }
                        while i as usize >= arr.len() {
                            arr.push(Default::default());
                        }
                        let mut scalar: Self = Default::default();
                        scalar.read(r)?;
                        arr.set(i as usize, scalar);
                    }
                    state.end_data().has_end_bytes(r)?;
                    return Ok(())
                }
            }
        }
        return Err(io::ErrorKind::InvalidData.into())
    }
}

impl Scalar for u8 {
    #[inline]
    fn ty() -> Type {Type::U8}
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        w.write(&[*self])
    }
    fn read<R: io::Read>(&mut self, r: &mut R) -> io::Result<usize> {
        let mut buf: [u8; 1] = [0; 1];
        r.read_exact(&mut buf)?;
        *self = buf[0];
        Ok(1)
    }
}

impl Scalar for u16 {
    #[inline]
    fn ty() -> Type {Type::U16}
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        let le = self.to_le();
        w.write(&[le as u8, (le >> 8) as u8])
    }
    fn read<R: io::Read>(&mut self, r: &mut R) -> io::Result<usize> {
        let mut buf: [u8; 2] = [0; 2];
        r.read_exact(&mut buf)?;
        *self = u16::from_le(buf[0] as u16 | (buf[1] as u16) << 8);
        Ok(2)
    }
}

impl Scalar for u32 {
    #[inline]
    fn ty() -> Type {Type::U32}
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        let le = self.to_le();
        w.write(&[le as u8, (le >> 8) as u8, (le >> 16) as u8, (le >> 24) as u8])
    }
    fn read<R: io::Read>(&mut self, r: &mut R) -> io::Result<usize> {
        let mut buf: [u8; 4] = [0; 4];
        r.read_exact(&mut buf)?;
        *self = u32::from_le(
            buf[0] as u32 | (buf[1] as u32) << 8 |
            (buf[2] as u32) << 16 | (buf[3] as u32) << 24
        );
        Ok(4)
    }
}

impl Scalar for u64 {
    #[inline]
    fn ty() -> Type {Type::U64}
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        let le = self.to_le();
        w.write(&[
            le as u8, (le >> 8) as u8, (le >> 16) as u8, (le >> 24) as u8,
            (le >> 32) as u8, (le >> 40) as u8, (le >> 48) as u8, (le >> 56) as u8
        ])
    }
    fn read<R: io::Read>(&mut self, r: &mut R) -> io::Result<usize> {
        let mut buf: [u8; 8] = [0; 8];
        r.read_exact(&mut buf)?;
        *self = u64::from_le(
            buf[0] as u64 | (buf[1] as u64) << 8 |
            (buf[2] as u64) << 16 | (buf[3] as u64) << 24 |
            (buf[4] as u64) << 32 | (buf[5] as u64) << 40 |
            (buf[6] as u64) << 48 | (buf[7] as u64) << 56
        );
        Ok(8)
    }
}

impl Scalar for i8 {
    #[inline]
    fn ty() -> Type {Type::I8}
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        w.write(&[*self as u8])
    }
    fn read<R: io::Read>(&mut self, r: &mut R) -> io::Result<usize> {
        let mut val: u8 = 0;
        let n = val.read(r)?;
        *self = val as i8;
        Ok(n)
    }
}

impl Scalar for i16 {
    #[inline]
    fn ty() -> Type {Type::I8}
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        (*self as u16).write(w)
    }
    fn read<R: io::Read>(&mut self, r: &mut R) -> io::Result<usize> {
        let mut val: u16 = 0;
        let n = val.read(r)?;
        *self = val as i16;
        Ok(n)
    }
}

impl Scalar for i32 {
    #[inline]
    fn ty() -> Type {Type::I8}
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        (*self as u32).write(w)
    }
    fn read<R: io::Read>(&mut self, r: &mut R) -> io::Result<usize> {
        let mut val: u32 = 0;
        let n = val.read(r)?;
        *self = val as i32;
        Ok(n)
    }
}

impl Scalar for i64 {
    #[inline]
    fn ty() -> Type {Type::I8}
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        (*self as u64).write(w)
    }
    fn read<R: io::Read>(&mut self, r: &mut R) -> io::Result<usize> {
        let mut val: u64 = 0;
        let n = val.read(r)?;
        *self = val as i64;
        Ok(n)
    }
}

impl Scalar for f32 {
    #[inline]
    fn ty() -> Type {Type::I8}
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        use std::mem::transmute;

        unsafe {transmute::<&f32, &u32>(self)}.write(w)
    }
    fn read<R: io::Read>(&mut self, r: &mut R) -> io::Result<usize> {
        use std::mem::transmute;

        let mut val: u32 = 0;
        let n = val.read(r)?;
        *self = unsafe {transmute::<u32, f32>(val)};
        Ok(n)
    }
}

impl Scalar for f64 {
    #[inline]
    fn ty() -> Type {Type::I8}
    fn write<W: io::Write>(&self, w: &mut W) -> io::Result<usize> {
        use std::mem::transmute;

        unsafe {transmute::<&f64, &u64>(self)}.write(w)
    }
    fn read<R: io::Read>(&mut self, r: &mut R) -> io::Result<usize> {
        use std::mem::transmute;

        let mut val: u64 = 0;
        let n = val.read(r)?;
        *self = unsafe {transmute::<u64, f64>(val)};
        Ok(n)
    }
}
