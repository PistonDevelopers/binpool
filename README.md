# binpool
An experimental uniform binary format for particle physics analysis

When running particle physics simulations,
it is sometimes handy to record the data and perform analysis
separately from the simulation process.

This binary format is designed to read and write particle
physics data to files or streams, such that the tools for analysis
can be reused with minimum setup.

Particle physics data is similar to video or animation streams
with the difference that time is arbitrarily defined.
This format allows precise control over data changes,
by changing the offset instance id and range.

10 built-in Rust number types are supported,
using array notation, with vector and matrix dimensions up to 80x80.
You can also define custom binary formats.

You can repeat the same data multiple times,
or write only ranges that changes.
Semantics is controlled by the application,
but the format is generic enough to reuse algorithms across projects.

### Format

```ignore
type format == 0 => end of stream
type format: u16, property id: u16
|- bytes == 0 => no more data, next property
|- bytes: u64, offset instance id: u64, data: [u8; bytes]
```

Integers are stored in little-endian format.

The number of items in the data are inferred from the number of bytes
and knowledge about the type format.

### Motivation for the design

A file format is uniform when it organized the same way everywhere.

One benefit with a uniform format is that you can easily split
data up into multiple files, or stream it across a network.
It is also easy to generate while running a physics simulation.

This file format assumes that the application has some
form of data structure, where each particle instance
is assigned a unique id, and their object relations
can be derived from a property id.
Since the property ids are known, one can use external tools
to analyze the physical properties relative to each other,
without any knowledge about the data structure.

To describe time, just create a new property id, e.g. f32 scalar,
and write out this value before other data in the same time step.

### Usage

The traits `Scalar`, `Vector` and `Matrix` are implemented for
array types of primitive integer and float formats.

When you write data to a file the order is preserved.

```ignore
use binpool::Scalar;

let prop_id = 0; // A unique property id.
let data: Vec<f32> = vec![1.0, 2.0, 3.0];
Scalar::write_array(prop_id, &data, &mut file).unwrap();
```

When you read from a file, e.g. to replay a recorded simulation,
you often read a single frame at a time then wait for the time to read next frame.
To do this, use a loop with flags for each property and break when
all read flags are set.

```ignore
use binpool::State;

let prop_id = 0; // A unique property id.
let mut read_prop_id = false;
let mut data: Vec<[f32; 2]> = vec![];
while let Ok((Some(state), ty, prop)) = State::read(&mut file) {
    match prop {
        prop_id if !read_prop_id => {
            Vector::read_array(state, ty, &mut data, &mut file).unwrap();
            read_prop_id = true;
        }
        _ => break,
    }
}
```

Data is often stored in a struct and overwritten for each frame.
The example above uses a local variable just for showing how to read data.

## License

Licensed under either of
 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you shall be dual licensed as above, without any
additional terms or conditions.
