extern crate nalgebra;

use std::ops::{Add,Mul};
use nalgebra::{Vector4, Vector3, Vector2, Norm, dot, cross};

// rendering. I think there should be three seperate layers: a raster layer at 2x resolution, a
// vector layer at 1x resloution, and a text layer at 1x resolution. all should have depth.

pub enum Pixel
{
    Color(f32, f32, f32),
    Grayscale(f32)
}

fn dither_2(val: usize, x: usize, y: usize) -> bool
{
    val > (2*y + 3*(x%2)) % 4
}

fn dither(value: f32, x: usize, y: usize) -> bool
{
    dither_2((value * 5.0).max(0.0).min(4.0) as usize, x, y)
}

pub fn to_256_color(p: &Pixel, x: usize, y: usize) -> usize
{
    0xE8 + match p {
        &Pixel::Grayscale(col) => {
            let val = (col * 24.25).max(0.0).min(24.24);
            if dither_2(((val - val.floor()) * 4.0) as usize, x, y) {
                val as usize + 1
            } else {
                val as usize
            }
        },
        _ => panic!("unimplimented")
    }
}

pub trait Varying
{
    fn combine(&Vec<(f32, &Self)>) -> Self;
}

impl <T> Varying for T where T:Add<Output=T> + Mul<f32, Output=T> + Clone
{
    fn combine(list: &Vec<(f32, &Self)>) -> Self
    {
        let mut acc = list[0].1.clone() * list[0].0;
        for i in 1..list.len() {
            acc = acc + list[i].1.clone() * list[i].0;
        }
        acc
    }
}

pub struct Buffer<T>
{
    pub width: usize,
    pub height: usize,
    buf: Vec<Option<(T,f32)>>
}

fn distribute(size: usize, pos: f32) -> i32
{
    ((pos * size as f32) + 1.0) as i32 - 1
}

impl <T> Buffer<T>
{
    pub fn new(width: usize, height: usize) -> Buffer<T>
    {
        let mut buf = Vec::with_capacity(width*height);
        for i in 0..(width*height) {
            buf.push(None);
        }
        Buffer {
            width: width,
            height: height,
            buf: buf
        }
    }
    fn get_index(&self, x: usize, y: usize) -> usize
    {
        self.width * y + x
    }
    pub fn apply(&mut self, x: usize, y:usize , (val, depth): (T, f32)) -> ()
    {
        let index = self.get_index(x, y);
        if let Some((_, d)) = self.buf[index] {
            if depth > d {
                self.buf[index] = Some((val, depth));
            }
        } else {
            self.buf[index] = Some((val, depth));
        }
    }
    fn ratio_to_xy(&self, (x, y): (f32, f32)) -> Option<(usize, usize)>
    {
        let ix = distribute(self.width, x);
        let iy = distribute(self.height, y);
        if 0 <= ix && ix < self.width as i32 && 0 <= iy && iy < self.height as i32 {
            Some((ix as usize, iy as usize))
        } else {
            None
        }
    }
    fn center_to_xy(&self, (x, y): (f32, f32)) -> Option<(usize, usize)>
    {
        self.ratio_to_xy(((x+1.0)/2.0, (y+1.0)/2.0))
    }
    pub fn get(&self, (x, y): (usize, usize)) -> &Option<(T, f32)>
    {
        &self.buf[self.get_index(x, y)]
    }
    pub fn clear(&mut self) -> ()
    {
        for i in 0..self.buf.len() {
            self.buf[i] = None;
        }
    }
}

pub enum Patch
{
    Point(usize),
    Line(usize, usize),
    Tri(usize, usize, usize)
}

impl Patch
{
    pub fn reverse(&self) -> Self {
        match self {
            &Patch::Point(x) => Patch::Point(x),
            &Patch::Line(a, b) => Patch::Line(b, a),
            &Patch::Tri(a, b, c) => Patch::Tri(c, b, a)
        }
    }
}

struct Vec4
{
    c: [f32; 4]
}
impl Varying for Vec4
{
    fn combine(list: &Vec<(f32, &Self)>) -> Self
    {
        Vec4 {
            c: list.iter().fold([0.0; 4], |acc, &(v, x)| {
                let mut thing = acc;
                for i in 0..4 {
                    thing[i] += v* x.c[i]
                }
                thing
            })
        }
    }
}

fn line_it((sx, sy): (i32, i32), (ex, ey): (i32, i32)) -> Box<Iterator<Item=(i32, i32, f32)>> {
    if sx == ex && sy == ey {
        Box::new((0..1).map(move |_|{ (sx, sy, 0.0) }))
    } else if (ex - sx).abs() >= (ey - sy).abs() {
        Box::new((0..((ex - sx).abs() + 1)).map(move |i| {
            (i*(ex-sx).signum() + sx, (i*(ey-sy))/(ex-sx).abs() + sy, i as f32 / (ex - sx).abs() as f32)
        }))
    } else {
        Box::new(line_it((sy, sx), (ey, ex)).map(|(y, x, v)| { (x, y, v) }))
    }
}

fn to_buffer_coord<T>(buf: &Buffer<T>, coord: &Vector4<f32>) -> Vector2<f32>
{
    Vector2::new(((coord.x+1.0) * (buf.width as f32) / 2.0) - 0.5, ((coord.y+1.0) * (buf.height as f32) / 2.0) - 0.5)
}

fn get_interp(target: Vector3<f32>, a: Vector3<f32>, b: Vector3<f32>, c: Vector3<f32>) -> (f32, f32, f32)
{
    let vecs = [a, b, c];
    let mut out = [0.0, 0.0, 0.0];
    for i in 0..3 {
        let (a, b, c) = (vecs[i], vecs[(i+1)%3], vecs[(i+2)%3]);
        out[i] = dot(&target, &cross(&b, &c)) / dot(&a, &cross(&b, &c));
    }
    (out[0], out[1], out[2])
}

pub fn process<V,U,T,E,F>(buf: &mut Buffer<T>, uniform: &U, varying: &Vec<V>, patches: &Vec<Patch>, vertex: E, fragment: F) -> ()
    where V:Varying, E: Fn(&U,&V) -> Vector4<f32>, F: Fn(&U,&V) -> Option<T>
{
    let mut varied = Vec::new();
    for point in varying {
        varied.push(vertex(uniform, point));
    }
    for patch in patches {
        match patch {
            &Patch::Point(index) => {
                let pos = varied[index];
                if let Some((x, y)) = buf.center_to_xy((pos.x, pos.y)) {
                    if let Some(val) = fragment(uniform, &varying[index]) {
                        buf.apply(x, y, (val, varied[index].z));
                    }
                }
            },
            &Patch::Line(i_a, i_b) => {
                let pos_a = varied[i_a];
                let pos_b = varied[i_b];
                if let (Some((ax, ay)), Some((bx,by))) = (buf.center_to_xy((pos_a.x,pos_a.y)), buf.center_to_xy((pos_b.x,pos_b.y))) {
                    for (x, y, d) in line_it((ax as i32,ay as i32),(bx as i32,by as i32)) {
                        //println!("{} {}", x, y);
                        let loc = Vector4::combine(&vec![(d, &varied[i_b]), (1.0 - d, &varied[i_a])]);
                        if let Some(val) = fragment(uniform, &V::combine(&vec![(d,&varying[i_b]),(1.0 - d, &varying[i_a])])) {
                            buf.apply(x as usize, y as usize, (val, loc.z));
                        }
                    }
                }
            },
            &Patch::Tri(i_a, i_b, i_c) => {
                // uhg. this is not going to be fun.
                // rule: points whose centers fall in the tri are rendered.
                // visible tries move clockwise (counterclockwise cartesian)
                render_tri(buf, uniform, &[varied[i_a].clone(), varied[i_b].clone(), varied[i_c].clone()], &[&varying[i_a], &varying[i_b], &varying[i_c]], &fragment);
            }
        }
    }
}

pub fn render_tri<T, U, V, F>(buf: &mut Buffer<T>, uniform: &U, verts: &[Vector4<f32>; 3], varying: &[&V; 3], fragment: &F) -> ()
    where V:Varying, F: Fn(&U,&V) -> Option<T>
{
    let mut norms = [Vector2::new(0.0,0.0); 3];
    let mut offsets = [0.0; 3];
    let mut denom = 0.0;
    let mut x = 0.0;
    let mut y = 0.0;
    for i in 0..3 {
        let (a,b,c) = (verts[i], verts[(i+1)%3], verts[(i+2)%3]);
        let t = (b.w*Vector2::new(c.x, c.y) - c.w*Vector2::new(b.x, b.y)).normalize();
        norms[i] = Vector2::new(t.y, -t.x);
        offsets[i] = - dot(&norms[i], &(b.w.recip()*Vector2::new(b.x, b.y)));
        denom += a.w*(c.x*b.y - b.x*c.y);
        x += a.w*(c.x-b.x);
        y += a.w*(c.y-b.y);
    }
    let vec = Vector2::new(-y/denom, x/denom);
    let tar = verts.iter().fold(verts[0], |a, b| { if a.z.abs() > b.z.abs() { a } else { b.clone()}});
    let num = (1.0 - dot(&vec, &Vector2::new(tar.x, tar.y)))/tar.w;

    let (a,b,c) = (
        Vector3::new(verts[0].x, verts[0].y, verts[0].w),
        Vector3::new(verts[1].x, verts[1].y, verts[1].w),
        Vector3::new(verts[2].x, verts[2].y, verts[2].w));

    for xi in 0..buf.width {
        for yi in 0..buf.height {
            let (x, y) = ((xi as f32/buf.width as f32), (yi as f32/buf.height as f32));
            let (x, y) = (x*2.0 -1.0, y*2.0-1.0);
            let screen = Vector2::new(x,y);

            let mut within = true;
            for i in 0..3 {
                if dot(&screen, &norms[i]) + offsets[i] > 0.0 {
                    within = false;
                    break;
                }
            }
            if within {
                let val = (dot(&vec, &screen)+num).recip();
                let pos = Vector3::new(screen.x*val, screen.y*val, val);
                let interp = get_interp(pos, a, b, c);
                let varied = V::combine(&vec![(interp.0,varying[0]), (interp.1,varying[1]), (interp.2,varying[2])]);
                if let Some(v) = fragment(uniform, &varied) {
                    buf.apply(xi, yi, (v, verts[0].z*interp.0 + verts[1].z*interp.1+ verts[2].z*interp.2));
                }
            }
        }
    }
}
