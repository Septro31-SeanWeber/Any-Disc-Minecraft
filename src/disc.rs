#[allow(dead_code)]
#[derive(Default)]
pub struct Disc{
    pub view_name: String,
    pub ogg_name: String,
    pub album: String
}

#[allow(dead_code)]
impl Disc{
    pub fn new(view_name: String, ogg_name: String, album: String) -> Self {
        Self {
            view_name,
            ogg_name,
            album
        }
    }
}