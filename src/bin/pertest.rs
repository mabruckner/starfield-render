extern crate starfield_render;
extern crate nalgebra;

use starfield_render as sf;

use nalgebra::{
    Vector4,
    Vector3,
    Vector2,
    dot
};

fn print_mat(buf: &sf::Buffer<sf::Pixel>)
{
    for y in 0..buf.height {
        for x in 0..buf.width {
            match buf.get((x,y)) {
                &Some((ref col, _)) => print!("\x1B[48;5;{}m ", sf::to_256_color(col, x, y)),
                &None => print!("\x1B[48;5;0m ")
            }
        }
        println!("");
    }
}

fn test_depth(buf: &mut sf::Buffer<sf::Pixel>, verts: &[Vector3<f32>; 3], mul: f32) -> ()
{
    let mut denom = 0.0;
    let mut x = 0.0;
    let mut y = 0.0;
    for i in 0..3 {
        let (a,b,c) = (verts[i], verts[(i+1)%3], verts[(i+2)%3]);
        denom += a.z*(c.x*b.y - b.x*c.y);
        x += a.z*(c.x-b.x);
        y += a.z*(c.y-b.y);
    }
    let vec = Vector2::new(-y/denom, x/denom);
    let tar = verts.iter().fold(verts[0], |a, b| { if a.z.abs() > b.z.abs() { a } else { b.clone()}});
    let num = (1.0 - dot(&vec, &Vector2::new(tar.x, tar.y)))/tar.z;
    for xi in 0..buf.width {
        for yi in 0..buf.height {
            let (x, y) = ((xi as f32/buf.width as f32), (yi as f32/buf.height as f32));
            let (x, y) = (x*2.0 -1.0, y*2.0-1.0);
            let val = (dot(&vec, &Vector2::new(x,y))+num).recip() * mul;
            buf.apply(xi, yi, (sf::Pixel::Grayscale(val - val.floor()), 0.0));
        }
    }
}

fn main()
{
    println!("TEST PROGRAM 2: GRADIENT");

    let (width, height) = (100, 50);

    let verts = [
        Vector3::new(-1.0, 0.0, 1.0),
        Vector3::new(0.0, 1.0, 1.0),
        Vector3::new(-1.0, 1.1, 2.0)];
    
    let mut buffer = sf::Buffer::new(width,height);

    buffer.clear();
    test_depth(&mut buffer, &verts, 1.0);
    print_mat(&buffer);
    buffer.clear();
    test_depth(&mut buffer, &verts, 0.5);
    print_mat(&buffer);
    buffer.clear();
    test_depth(&mut buffer, &verts, 0.1);
    print_mat(&buffer);
}
