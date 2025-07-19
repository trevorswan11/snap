use crate::img::crop::CropMethod;
use crate::img::scale::ScaleMethod;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "snap",
    version,
    author = "Trevor Swan",
    about = "Image processing"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: TopLevelCommand,
}

#[derive(Subcommand)]
pub enum TopLevelCommand {
    #[command(subcommand)]
    Img(ImgCommand),
}

#[derive(Subcommand)]
pub enum ImgCommand {
    #[command(about = "Resizes the image to the new height and width")]
    Resize {
        filepath_in: String,
        filepath_out: String,
        new_width: usize,
        new_height: usize,
        method: ScaleMethod,

        #[arg(long, required = false)]
        crop_x: Option<CropMethod>,

        #[arg(long, required = false)]
        crop_y: Option<CropMethod>,
    },

    #[command(about = "Scales the image up to the new height and width")]
    Scale {
        filepath_in: String,
        filepath_out: String,
        new_width: usize,
        new_height: usize,

        #[arg(long, required = false)]
        method: Option<ScaleMethod>,
    },

    #[command(about = "Crops the image down to the new height and width")]
    Crop {
        filepath_in: String,
        filepath_out: String,
        new_width: usize,
        new_height: usize,
        #[arg(long, required = false)]
        method: Option<CropMethod>,
    },

    #[command(
        about = "Applies seam carving to the image to reach the new height and width",
        alias = "sc"
    )]
    SeamCarve {
        filepath_in: String,
        filepath_out: String,
        new_width: usize,
        new_height: usize,
    },

    #[command(about = "Multiplies each pixel by the given scalars", alias = "tint")]
    ScaleRGB {
        filepath_in: String,
        filepath_out: String,
        r_scale: f64,
        g_scale: f64,
        b_scale: f64,
    },

    #[command(
        about = "Applies a hue shift wrapping the given number of degrees",
        alias = "hue"
    )]
    HueShift {
        filepath_in: String,
        filepath_out: String,
        degrees: f64,
    },

    #[command(
        about = "Converts any supported image to the output file specified",
        alias = "save"
    )]
    Convert {
        filepath_in: String,
        filepath_out: String,
    },
}
