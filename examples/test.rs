extern crate binpool;

use binpool::{Scalar, Vector, Matrix, State};
use std::fs::File;

fn main() {
    scalar();
    vector();
    matrix();
}

const ARRAY_PROPERTY: u16 = 3;
const SINGLE_PROPERTY: u16 = 4;

fn scalar() {
    let filename = "assets/test-scalar.pool";

    let mut file = File::create(filename).unwrap();
    let data: Vec<f32> = vec![1.0, 2.0, 3.0];
    Scalar::write_array(ARRAY_PROPERTY, &data, &mut file).unwrap();
    (10 as u8).write_property(SINGLE_PROPERTY, &mut file).unwrap();
    drop(file);

    let mut file = File::open(filename).unwrap();

    let mut data: Vec<f32> = vec![];
    let mut val: u8 = 0;
    while let Ok((Some(state), ty, prop)) = State::read(&mut file) {
        match prop {
            ARRAY_PROPERTY => Scalar::read_array(state, ty, &mut data, &mut file).unwrap(),
            SINGLE_PROPERTY => val.read_property(state, ty, &mut file).unwrap(),
            _ => break,
        }
    }

    println!("=== Scalar ===");
    println!("data {:?}", data);
    println!("val {:?}", val);
}

fn vector() {
    let filename = "assets/test-vector.pool";

    let mut file = File::create(filename).unwrap();
    let data: Vec<[f32; 2]> = vec![[1.0, 2.0], [3.0, 4.0]];
    Vector::write_array(ARRAY_PROPERTY, &data, &mut file).unwrap();
    let val: [u8; 2] = [10; 2];
    val.write_property(SINGLE_PROPERTY, &mut file).unwrap();
    drop(file);

    let mut file = File::open(filename).unwrap();
    let mut data: Vec<[f32; 2]> = vec![];
    let mut val: [u8; 2] = [0; 2];
    while let Ok((Some(state), ty, prop)) = State::read(&mut file) {
        match prop {
            ARRAY_PROPERTY => Vector::read_array(state, ty, &mut data, &mut file).unwrap(),
            SINGLE_PROPERTY => val.read_property(state, ty, &mut file).unwrap(),
            _ => break,
        }
    }

    println!("=== Vector ===");
    println!("data {:?}", data);
    println!("val {:?}", val);
}

fn matrix() {
    let filename = "assets/test-matrix.pool";

    let mut file = File::create(filename).unwrap();
    let data: Vec<[[f32; 2]; 2]> = vec![[[1.0, 2.0], [3.0, 4.0]]];
    Matrix::write_array(ARRAY_PROPERTY, &data, &mut file).unwrap();
    let val: [[u8; 2]; 2] = [[10; 2]; 2];
    val.write_property(SINGLE_PROPERTY, &mut file).unwrap();
    drop(file);

    let mut file = File::open(filename).unwrap();
    let mut data: Vec<[[f32; 2]; 2]> = vec![];
    let mut val: [[u8; 2]; 2] = [[0; 2]; 2];

    while let Ok((Some(state), ty, prop)) = State::read(&mut file) {
        match prop {
            ARRAY_PROPERTY => Matrix::read_array(state, ty, &mut data, &mut file).unwrap(),
            SINGLE_PROPERTY => val.read_property(state, ty, &mut file).unwrap(),
            _ => break,
        }
    }

    println!("=== Matrix ===");
    println!("data {:?}", data);
    println!("val {:?}", val);
}
