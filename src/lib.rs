extern crate nalgebra;

mod buffer;
mod render;
mod text;

pub use buffer::*;
pub use render::*;
pub use text::*;
use std::ops::{Add,Mul};
use nalgebra::{Vector4, Vector3, Vector2, Norm, dot, cross};

#[derive(Copy, Clone, Debug)]
pub enum Pixel
{
    Color(f32, f32, f32),
    Grayscale(f32)
}

// I always relish the opportunity to place what looks like indecipherable alien symbology in my
// code.
static blocks: [char; 16] = [' ','▘','▝','▀','▖','▌','▞','▛','▗','▚','▐','▜','▄','▙','▟','█'];

pub fn grid_cell(buf: &Buffer<bool>, x: usize, y:usize) -> char {
    let (ul, ur, ll, lr) = (buf.get(x,y+1), buf.get(x+1, y+1), buf.get(x, y), buf.get(x+1, y));
    let index = if *ul{1}else{0} + if *ur{2}else{0} + if *ll{4}else{0} + if *lr{8}else{0};
    blocks[index]
}

fn raster_to_char(ch: &mut Buffer<ColorChar>, buf: &DepthBuffer<Pixel>)
{
    for y in 0..buf.height {
        for x in 0..buf.width {
            ch.set(x, y, ColorChar(7, match buf.get(x, y) {
                &Some((ref val, _)) => to_256_color(val, x, y) as u8,
                &None => 0
            }, ' '));
        }
    }
}

fn dither_2(val: usize, x: usize, y: usize) -> bool
{
    val > (2*y + 3*(x%2)) % 4
}

fn dither(value: f32, x: usize, y: usize) -> bool
{
    dither_2((value * 5.0).max(0.0).min(4.0) as usize, x, y)
}

pub fn to_256_color(p: &Pixel, x: usize, y: usize) -> u8
{
    match p {
        &Pixel::Grayscale(col) => {
            let val = (col * 24.25).max(0.0).min(24.24);
            let res = 0xE8 + if dither_2(((val - val.floor()) * 4.0) as usize, x, y) {
                val as usize + 1
            } else {
                val as usize
            };
            if res > 255 {
                0xf
            } else {
                res as u8
            }
        },
        _ => panic!("unimplimented")
    }
}

