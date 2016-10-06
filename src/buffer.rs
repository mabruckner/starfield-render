use std::slice;

pub struct Rect{
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize
}

pub struct Buffer<T>
{
    pub width: usize,
    pub height: usize,
    buf: Vec<T>
}



fn distribute(size: usize, pos: f32) -> i32
{
    ((pos * size as f32) + 1.0) as i32 - 1
}

impl <T: Copy> Buffer<T>
{
    pub fn new(width: usize, height: usize, val: T) -> Buffer<T>
    {
        let mut buf = Vec::with_capacity(width*height);
        for i in 0..(width*height) {
            buf.push(val);
        }
        Buffer {
            width: width,
            height: height,
            buf: buf
        }
    }
    pub fn fill(&mut self, val: T) -> ()
    {
        for i in 0..self.buf.len() {
            self.buf[i] = val;
        }
    }
}

impl <T> Buffer<T>
{
    fn get_index(&self, x: usize, y: usize) -> usize
    {
        self.width * y + x
    }
    pub fn set(&mut self, x: usize, y:usize , val: T) -> ()
    {
        let index = self.get_index(x, y);
        self.buf[index] = val;
    }
    pub fn ratio_to_xy(&self, x: f32, y: f32) -> Option<(usize, usize)>
    {
        let ix = distribute(self.width, x);
        let iy = distribute(self.height, y);
        if 0 <= ix && ix < self.width as i32 && 0 <= iy && iy < self.height as i32 {
            Some((ix as usize, iy as usize))
        } else {
            None
        }
    }
    pub fn center_to_xy(&self, x: f32, y: f32) -> Option<(usize, usize)>
    {
        self.ratio_to_xy((x+1.0)/2.0, (y+1.0)/2.0)
    }
    pub fn get(&self, x: usize, y: usize) -> &T
    {
        &self.buf[self.get_index(x, y)]
    }
    pub fn row_iter<'a>(&'a self, y: usize) -> slice::Iter<'a, T>
    {
        self.buf[y*self.width .. (y+1)*self.width].iter()
    }
    pub fn get_rect(&self) -> Rect
    {
        Rect{
            x: 0,
            y: 0,
            w: self.width,
            h: self.height
        }
    }
}

pub type DepthBuffer<T> = Buffer<Option<(T, f32)>>;

impl <T> DepthBuffer<T>
{
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
    pub fn clear(&mut self) -> ()
    {
        for i in 0..self.buf.len() {
            self.buf[i] = None;
        }
    }
}
