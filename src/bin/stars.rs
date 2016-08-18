extern crate starfield_render;
extern crate rand;
extern crate nalgebra;

use starfield_render as sf;
use rand::distributions::{IndependentSample, Range};
use std::f32;

use nalgebra::{
    Vector3,
    Vector4,
    Rotation3
};

fn print_buffer(buf: &sf::Buffer<char>) -> ()
{
    for y in 0..buf.height {
        for x in 0..buf.width {
            if let &Some((c, _)) = buf.get((x,y)) {
                print!("{}", c);
            } else {
                print!(" ");
            }
        }
        println!("");
    }
}

fn main() -> ()
{
    println!("TEST PROGRAM 1: STARS");

    let (width, height) = (100, 50);

    let mut patches = Vec::new();
    let mut verts: Vec<Vector4<f32>> = Vec::new();

    let mut rng = rand::thread_rng();
    let mut range = Range::new(-0.5, 0.5);
    for i in 0..100 {
        patches.push(sf::Patch::Point(i));
        verts.push(Vector4::new(range.ind_sample(&mut rng), range.ind_sample(&mut rng), range.ind_sample(&mut rng), 0.0));
    }
    let mut range = Range::new(0, verts.len());
    let mut lines = Vec::new();
    for i in 0..10 {
        lines.push(sf::Patch::Line(range.ind_sample(&mut rng), range.ind_sample(&mut rng)));
    }
    let verts_tri: Vec<Vector4<f32>> = vec![Vector4::new(0.0, 0.0, 0.0, 0.0), Vector4::new(1.0, -0.5, 0.0, 1.0), Vector4::new(0.5, 1.0, 0.0, 0.0)];
    let mut tris = vec![sf::Patch::Tri(0, 1, 2)];
    let reversed = tris[0].reverse();
    tris.push(reversed);

    let mut buffer = sf::Buffer::new(width, height);

    let vertex = |u: &f32, v: &Vector4<f32>| {
        Vector4::new(v.x * u.cos() + v.z* u.sin(), v.y, v.z * u.cos() - v.x* u.sin(), 1.0)
    };

    let fragment = |u: &f32, v: &Vector4<f32>| {
        Some('X')
    };
    let fragment_dots = |u: &f32, v: &Vector4<f32>| {
        Some(':')
    };
    let fragment_points = |u: &f32, v: &Vector4<f32>| {
        Some('.')
    };

    let mut val = 0.0;
    loop {
        val += 0.001;
        print_buffer(&buffer);
        buffer.clear();
        sf::process(&mut buffer, &val, &verts, &patches, &vertex, &fragment);
        //sf::process(&mut buffer, &val, &verts, &lines, &vertex, &fragment_dots);
        sf::process(&mut buffer, &val, &verts_tri, &tris, &vertex, &fragment_dots);
        println!("\x1B[{}A", height+1);
    }
}
