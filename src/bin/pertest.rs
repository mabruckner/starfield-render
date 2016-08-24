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

fn main()
{
    println!("TEST PROGRAM 3: PERSPECTIVE");

    let (width, height) = (100, 50);

    let verts = vec![
        Vector4::new(-1.0, 1.0, 2.0, 2.0),
        Vector4::new(-1.0, 0.0, 2.0, 2.0),
        Vector4::new(-1.0, 0.0, 3.0, 3.0)];

    let dat = vec![
        Vector2::new(1.0, -1.0),
        Vector2::new(-2.0, 1.0),
        Vector2::new(1.0, 1.0)];
    
    let patches = vec![sf::Patch::Tri(0,1,2)];

    let mut buffer = sf::Buffer::new(width,height);

    let fragment = |u: &f32, v: &Vector2<f32>| {
        let v = v.x.hypot(v.y);
        if v < 1.0 {
            Some(sf::Pixel::Grayscale((0.5+u.cos()/2.0)*(1.0 - v)))
        } else {
            Some(sf::Pixel::Grayscale(0.5))
        }
    };

    buffer.clear();
    sf::render(&mut buffer, &0.0, &verts, &dat, &patches, &fragment);
    print_mat(&buffer);
}
