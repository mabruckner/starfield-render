use buffer::Buffer;


#[derive(Copy,Clone)]
pub struct ColorChar(pub u8, pub u8, pub char);

pub fn make_colorstring<I>(it: I) -> String
    where I: Iterator<Item = ColorChar>
{
    let mut st = String::new();
    let mut pfg = None;
    let mut pbg = None;
    for ColorChar(fg, bg, ch) in it {
        if pfg != Some(fg) {
            st.push_str(&format!("\x1B[38;5;{}m", fg));
            pfg = Some(fg);
        }
        if pbg != Some(bg) {
            st.push_str(&format!("\x1B[48;5;{}m", bg));
            pbg = Some(bg);
        }
        st.push(ch);
    }
    st
}

/// A trait for objects that can be represented as a grid of characters.
pub trait GridPrint {
    /// Gets the dimensions as (width, height). Calls to `get_cell` should not exceed these bounds.
    fn get_size(&self) -> (usize, usize);

    /// Gets the character residing in cell (x,y). Calls to `get_cell` should respect the bounds set
    /// by `get_size`. This library assumes that y index increases with decreasing vertical position.
    /// (This is contrary to starfield)
    fn get_cell(&self, x:usize, y:usize) -> ColorChar;

    /// Print the grid to standard out.
    fn print(&self) {
        let (width,height) = self.get_size();
        for i in 0..height {
            println!("{}", make_colorstring((0..width).map(|x|{self.get_cell(x, i)})));
        }
    }
}

impl GridPrint for Buffer<ColorChar>
{
    fn get_size(&self) -> (usize, usize) {
        (self.width, self.height)
    }

    fn get_cell(&self, x:usize, y:usize) -> ColorChar {
        self.get(x, self.height-y-1).clone()
    }
}
