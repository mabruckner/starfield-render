
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
