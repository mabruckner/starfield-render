extern crate nalgebra;

use std::ops::{Add,Mul};
use nalgebra::{Vector4, Vector2, Norm, dot};

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
            let val = (col * 24.0).max(0.0).min(23.99);
            if dither(val - val.floor(), x, y) {
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
    fn apply(&mut self, x: usize, y:usize , (val, depth): (T, f32)) -> ()
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

fn get_interp(target: Vector2<f32>, a: Vector2<f32>, b: Vector2<f32>, c: Vector2<f32>) -> (f32, f32, f32)
{
    let d_b = b-a;
    let d_c = c-a;
    let d = target-a;
//    let right = Vector2::new(-d_b.y, d_b.x).normalize();
//    let c_v = 1.0 - dot(&right, &d) / dot(&right, &c);
//    let b_v = 1.0 - (d - (c_v * d_c)).norm() / (d_b).norm();
    let denom = d_b.x*d_c.y - d_b.y*d_c.x;
    let c_v = (d.y*d_b.x - d.x*d_b.y) / denom;
    let b_v = (d.x*d_c.y - d.y*d_c.x) / denom;
    (1.0 - c_v - b_v, b_v, c_v)
}

pub fn process<V,U,T,E,F>(buf: &mut Buffer<T>, uniform: &U, varying: &Vec<V>, patches: &Vec<Patch>, vertex: E, fragment: F) -> ()
    where V:Varying, E: Fn(&U,&V) -> Vector4<f32>, F: Fn(&U,&V,&Vector4<f32>) -> Option<(T, f32)>
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
                    if let Some(val) = fragment(uniform, &varying[index], &varied[index]) {
                        buf.apply(x, y, val);
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
                        if let Some(val) = fragment(uniform, &V::combine(&vec![(d,&varying[i_b]),(1.0 - d, &varying[i_a])]), &loc) {
                            buf.apply(x as usize, y as usize, val);
                        }
                    }
                }
            },
            &Patch::Tri(i_a, i_b, i_c) => {
                // uhg. this is not going to be fun.
                // rule: points whose centers fall in the tri are rendered.
                // visible tries move clockwise (counterclockwise cartesian)
                let pos_a = to_buffer_coord(&buf, &varied[i_a]);
                let pos_b = to_buffer_coord(&buf, &varied[i_b]);
                let pos_c = to_buffer_coord(&buf, &varied[i_c]);
                // permute
                let (pos_a, pos_b, pos_c, i_a, i_b, i_c) = if pos_b.x < pos_a.x && pos_b.x < pos_c.x {
                    (pos_b, pos_c, pos_a, i_b, i_c, i_a)
                } else if pos_c.x < pos_a.x && pos_c.x < pos_b.x {
                    (pos_c, pos_a, pos_b, i_c, i_a, i_b)
                } else {
                    (pos_a, pos_b, pos_c, i_a, i_b, i_c)
                };

                let delta_ab = pos_b - pos_a;
                let delta_ac = pos_c - pos_a;
                let w1 = delta_ab.x.min(delta_ac.x);
                if w1 != 0.0 {
                    let m1 = delta_ab.y / delta_ab.x;
                    let m2 = delta_ac.y / delta_ac.x;
                    if m1 < m2 {
                        for x in (pos_a.x.ceil().max(0.0) as i32)..((pos_a.x+w1).floor().min(buf.width as f32 -1.0) as i32 + 1) {
                            let dx = x as f32 - pos_a.x;
                            for y in ((pos_a.y + m1*dx).ceil().max(0.0) as i32)..((pos_a.y + m2*dx).floor().min(buf.height as f32 -1.0) as i32 + 1) {
                                let (fa, fb, fc) = get_interp(Vector2::new(x as f32, y as f32), pos_a, pos_b, pos_c);
                                let combined = V::combine(&vec![(fa, &varying[i_a]), (fb, &varying[i_b]), (fc, &varying[i_c])]);
                                let loc = Vector4::combine(&vec![(fa, &varied[i_a]), (fb, &varied[i_b]), (fc, &varied[i_c])]);
                                if let Some(val) = fragment(uniform, &combined, &loc) {
                                    buf.apply(x as usize, y as usize, val);
                                }
                            }
                        }
                    }
                }
                let (pos_r, pos_top, pos_bot, i_r, i_top, i_bot) = if pos_b.x > pos_c.x {
                    (pos_b, pos_c, pos_a, i_b, i_c, i_a)
                } else {
                    (pos_c, pos_a, pos_b, i_c, i_a, i_b)
                };
                let delta_top = pos_r - pos_top;
                let delta_bot = pos_r - pos_bot;
                let w2 = delta_top.x.min(delta_bot.x);
                if w2 != 0.0 {
                    let mt = delta_top.y / delta_top.x;
                    let mb = delta_bot.y / delta_bot.x;
                    if mb > mt {
                        for x in ((pos_r.x - w2).ceil().max(0.0) as i32)..(pos_r.x.floor().min(buf.width as f32 -1.0) as i32 +1) {
                            let dx = x as f32 - pos_r.x;
                            for y in ((pos_r.y + mb*dx).ceil().max(0.0) as i32)..((pos_r.y + mt*dx).floor().min(buf.height as f32 -1.0) as i32 + 1) {
                                let (fr, fb, ft) = get_interp(Vector2::new(x as f32, y as f32), pos_r, pos_bot, pos_top);
                                let combined = V::combine(&vec![(fr, &varying[i_r]), (fb, &varying[i_bot]), (ft, &varying[i_top])]);
                                let loc = Vector4::combine(&vec![(fr, &varied[i_r]), (fb, &varied[i_bot]), (ft, &varied[i_top])]);
                                if let Some(val) = fragment(uniform, &combined, &loc) {
                                    buf.apply(x as usize, y as usize, val);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}


// vertex \
