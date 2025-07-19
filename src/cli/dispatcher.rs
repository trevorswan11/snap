use crate::cli::commands::*;
use crate::img::image::Image;
use crate::img::io::convert;

use clap::Parser;

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        TopLevelCommand::Img(img_cmd) => match img_cmd {
            ImgCommand::Resize {
                filepath_in,
                filepath_out,
                new_width,
                new_height,
                method,
                crop_x,
                crop_y,
            } => {
                let mut i = Image::from_file(&filepath_in)?;
                i.resize(new_width, new_height, method, crop_x, crop_y);
                i.save(&filepath_out)?;
            }
            ImgCommand::Scale {
                filepath_in,
                filepath_out,
                new_width,
                new_height,
                method,
            } => {
                let mut i = Image::from_file(&filepath_in)?;
                i.scale(
                    new_width,
                    new_height,
                    method.unwrap_or(crate::img::scale::ScaleMethod::Bilinear),
                );
                i.save(&filepath_out)?;
            }
            ImgCommand::Crop {
                filepath_in,
                filepath_out,
                new_width,
                new_height,
                method,
            } => {
                let mut i = Image::from_file(&filepath_in)?;
                i.crop(
                    new_width,
                    new_height,
                    method.unwrap_or(crate::img::crop::CropMethod::Rectangular),
                );
                i.save(&filepath_out)?;
            }
            ImgCommand::SeamCarve {
                filepath_in,
                filepath_out,
                new_width,
                new_height,
            } => {
                let mut i = Image::from_file(&filepath_in)?;
                i.seam_carve(new_width, new_height);
                i.save(&filepath_out)?;
            }
            ImgCommand::ScaleRGB {
                filepath_in,
                filepath_out,
                r_scale,
                g_scale,
                b_scale,
            } => {
                let mut i = Image::from_file(&filepath_in)?;
                i.scale_rgb(r_scale, g_scale, b_scale)?;
                i.save(&filepath_out)?;
            }
            ImgCommand::HueShift {
                filepath_in,
                filepath_out,
                degrees,
            } => {
                let mut i = Image::from_file(&filepath_in)?;
                i.hue_shift(degrees)?;
                i.save(&filepath_out)?;
            }
            ImgCommand::Convert {
                filepath_in,
                filepath_out,
            } => {
                convert(&filepath_in, &filepath_out)?;
            }
        },
    }

    Ok(())
}
