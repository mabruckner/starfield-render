extern crate starfield_render;
extern crate nalgebra;

use starfield_render as sf;

use nalgebra::{
    Vector4,
    Vector2
};

fn print_mat(buf: &sf::DepthBuffer<sf::Pixel>)
{
    for y in 0..buf.height {
        for x in 0..buf.width {
            match buf.get(x,y) {
                &Some((ref col, _)) => print!("\x1B[48;5;{}m ", sf::to_256_color(col, x, y)),
                &None => print!("\x1B[48;5;0m ")
            }
        }
        println!("");
    }
}

fn main()
{
    println!("TEST PROGRAM 2: GRADIENT");

    let (width, height) = (100, 50);

    let verts = vec![Vector2::new(-1.0, -1.0), Vector2::new(1.0, -1.0), Vector2::new(-1.0, 1.0), Vector2::new(1.0, 1.0)];
    let faces = vec![sf::Patch::Tri(0,1,2), sf::Patch::Tri(2,1,3)];

    let mut buffer = sf::Buffer::new(width,height,None);

    let vertex = |u: &f32, v: &Vector2<f32>| {
        (Vector4::new(v.x, v.y, 0.0, 1.0), v.clone())
    };

    let fragment = |u: &f32, v: &Vector2<f32>| {
        let v = v.x.hypot(v.y);
        if v < 1.0 {
            Some(sf::Pixel::Grayscale((0.5+u.cos()/2.0)*(1.0 - v)))
        } else {
            None
        }
    };

    let mut val = 0.0;
    loop {
        val += 0.004;
        print_mat(&buffer);
        buffer.clear();
        sf::process(&mut buffer, &val, &verts, &faces, &vertex, &fragment);
        println!("\x1B[{}A", height+1);
    }
}
