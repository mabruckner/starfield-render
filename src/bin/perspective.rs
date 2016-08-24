extern crate starfield_render;
extern crate nalgebra;

use starfield_render as sf;

use nalgebra::{
    Vector4,
    Vector3,
    Vector2,
    dot,
    Rotate,
    Rotation3
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

fn main()
{
    println!("TEST PROGRAM 3: PERSPECTIVE");

    let (width, height) = (100, 50);

    let verts = vec![
        Vector2::new(-1.0, -1.0),
        Vector2::new(1.0, -1.0),
        Vector2::new(-1.0, 1.0),
        Vector2::new(1.0, 1.0)];
    
    let patches = vec![sf::Patch::Tri(0,1,2), sf::Patch::Tri(3,2,1), sf::Patch::Tri(2,1,0), sf::Patch::Tri(1,2,3)];

    let mut buffer = sf::Buffer::new(width,height);

    let vertex = |u: &f32, v: &Vector2<f32>| {
        let p = Rotation3::new(Vector3::new(0.0, *u, 0.0)).rotate(&Vector3::new(v.x, v.y, 0.0));
        (Vector4::new(p.x, p.y, p.z, p.z+1.5), v.clone())
    };

    let fragment = |u: &f32, v: &Vector2<f32>| {
        let v = v.x.hypot(v.y);
        if v < 1.0 {
            Some(sf::Pixel::Grayscale((0.5+u.cos()/2.0)*(1.0 - v)))
        } else {
            Some(sf::Pixel::Grayscale(0.5))
        }
    };

    let mut val = 0.0;

    loop {
        val += 0.01;
        buffer.clear();
        sf::process(&mut buffer, &val, &verts, &patches, &vertex, &fragment);
        print_mat(&buffer);
        println!("\x1B[{}A", height+1);
    }
}