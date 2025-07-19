use std::error::Error;

mod seam {
    pub mod image;
    pub mod matrix;
}

mod utils {
    pub mod convert;
}

use crate::seam::image::Image;

fn main() -> Result<(), Box<dyn Error>> {
    let horse_in = "examples/horses.ppm";
    let horse_out = "examples/horses.jpg";
    // let horse_out_carved_ppm = "examples/horses_carved_300x382.ppm";
    // let horse_out_carved_img = "examples/horses_carved_300x382.jpg";
    let horse_out_carved_ppm = "examples/horses_carved_400x250.ppm";
    let horse_out_carved_img = "examples/horses_carved_400x250.jpg";

    utils::convert::ppm_to_jpeg(horse_in, horse_out)?;
    let mut i = Image::from_file(horse_in)?;
    i.seam_carve(400, 250);
    i.write_ppm_file(horse_out_carved_ppm)?;
    utils::convert::ppm_to_jpeg(horse_out_carved_ppm, horse_out_carved_img)?;
    Ok(())
}
