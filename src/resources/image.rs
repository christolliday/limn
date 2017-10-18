use image::{self, GenericImage};
use error as LimnError;

/// Image descriptor
#[derive(Debug, Clone)]
pub struct Image {
    pub info: ImageDescriptor,
    pub data: ImageData,
}

impl ImageInfo {

    pub fn from_file<P>(path: AsRef<Path>) 
                    -> Result<Self, LimnError> 
   {
        let image = image::open(path);
        let image_dims = image.dimensions();
        let format = match image {
            image::ImageLuma8(_) => ImageFormat::A8,
            image::ImageRgb8(_) => ImageFormat::RGB8,
            image::ImageRgba8(_) => ImageFormat::BGRA8,
            image::ImageLumaA8(_) => {
                return Err(image::ImageError::UnsupportedError("ImageLumaA8 unsupported".to_string()));
            }
        };
        
        let mut bytes = image.raw_pixels();
        if format == ImageFormat::BGRA8 {
            premultiply(bytes.as_mut_slice());
        }
        
        let opaque = is_image_opaque(format, &bytes[..]);
        let descriptor = ImageDescriptor::new(image_dims.0, image_dims.1, format, opaque);
        let data = ImageData::new(bytes);
        Ok((data, descriptor))
    }
    
    #[cfg(font_loader)]
    pub fn from_font_loader() 
        -> Result<Self, LimnError> 
    {
        
    }
}

fn is_image_opaque(format: ImageFormat, bytes: &[u8]) -> bool {
    match format {
        ImageFormat::BGRA8 => {
            let mut is_opaque = true;
            for i in 0..(bytes.len() / 4) {
                if bytes[i * 4 + 3] != 255 {
                    is_opaque = false;
                    break;
                }
            }
            is_opaque
        }
        ImageFormat::RGB8 => true,
        ImageFormat::RG8 => true,
        ImageFormat::A8 => false,
        ImageFormat::Invalid | ImageFormat::RGBAF32 => unreachable!(),
    }
}

// From webrender/wrench
// These are slow. Gecko's gfx/2d/Swizzle.cpp has better versions
fn premultiply(data: &mut [u8]) {
    for pixel in data.chunks_mut(4) {
        let a = pixel[3] as u32;
        let r = pixel[2] as u32;
        let g = pixel[1] as u32;
        let b = pixel[0] as u32;

        pixel[3] = a as u8;
        pixel[2] = ((r * a + 128) / 255) as u8;
        pixel[1] = ((g * a + 128) / 255) as u8;
        pixel[0] = ((b * a + 128) / 255) as u8;
    }
}
