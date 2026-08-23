#[allow(dead_code)]
#[derive(Default)]
pub struct Album {
    pub name: String,
    pub png_name: String,
}

#[allow(dead_code)]
impl Album {
    pub fn new(name: String, png_name: String) -> Self {
        Self { name, png_name }
    }
}
