use rusttype::Font;

pub struct FontSet {
    pub regular: Font<'static>,
    pub bold: Font<'static>,
}

impl FontSet {
    pub fn new(regular_data: Vec<u8>, bold_data: Vec<u8>) -> Self {
        Self {
            regular: Font::try_from_vec(regular_data).expect("Error loading regular font"),
            bold: Font::try_from_vec(bold_data).expect("Error loading bold font"),
        }
    }

    pub fn get(&self, weight: u16) -> &Font<'static> {
        if weight >= 700 {
            &self.bold
        } else {
            &self.regular
        }
    }
}
