use std::collections::HashMap;
use std::path::PathBuf;

use webrender::api::{RenderApi, ResourceUpdates, ExternalImageId, ExternalImageData, ImageKey, ImageFormat, ImageData, ImageDescriptor};
use image;
use image::ImageError;


#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum ImageSource {
    AbsolutePath(PathBuf),
    AssetPath(PathBuf),
    Bundled(String),
}

impl ImageSource {
    pub fn absolute<P: Into<PathBuf>>(path: P) -> Self {
        ImageSource::AbsolutePath(path.into())
    }
    pub fn asset<P: Into<PathBuf>>(path: P) -> Self {
        ImageSource::AssetPath(path.into())
    }
    pub fn bundled<P: Into<String>>(name: P) -> Self {
        ImageSource::Bundled(name.into())
    }
}

#[derive(Debug, Clone)]
pub struct ImageInfo {
    pub key: ImageKey,
    pub descriptor: ImageDescriptor,
}

#[derive(Default)]
pub struct ImageLoader {
    pub render: Option<RenderApi>,
    pub assets_path: PathBuf,
    pub bundled_images: HashMap<ImageSource, ImageInfo>,
    pub images: HashMap<ImageSource, ImageInfo>,
    pub texture_descriptors: HashMap<u64, ImageDescriptor>,
}

impl ImageLoader {
    pub fn new() -> Self {
        ImageLoader::default()
    }

    pub fn get_image(&mut self, source: &ImageSource) -> &ImageInfo {
        if self.images.contains_key(source) {
            &self.images[source]
        } else {
            let (data, descriptor) = match *source {
                ImageSource::AbsolutePath(ref path) => {
                    load_image_from_file(path).unwrap()
                },
                ImageSource::AssetPath(ref relative_path) => {
                    let mut path = PathBuf::from(&self.assets_path);
                    path.push(relative_path);
                    load_image_from_file(&path).unwrap()
                },
                ImageSource::Bundled(ref name) => {
                    panic!("Missing bundle image {}", name)
                }
            };

            self.put_image(source, data, descriptor)
        }
    }

    fn put_image(&mut self, source: &ImageSource, data: ImageData, descriptor: ImageDescriptor) -> &ImageInfo {
        let image_info = self.create_image_resource(data, descriptor);
        self.images.insert(source.clone(), image_info);
        &self.images[source]
    }

    pub fn create_image_resource(&mut self, data: ImageData, descriptor: ImageDescriptor) -> ImageInfo {
        let key = self.render_api().generate_image_key();
        let mut resources = ResourceUpdates::new();
        resources.add_image(key, descriptor, data, None);
        self.render_api().update_resources(resources);
        ImageInfo { key: key, descriptor: descriptor }
    }

    pub fn update_texture(&mut self, key: ImageKey, descriptor: ImageDescriptor, data: ExternalImageData) {
        let mut resources = ResourceUpdates::new();
        resources.update_image(key, descriptor, ImageData::External(data), None);
        self.render_api().update_resources(resources);
        let ExternalImageData { id: ExternalImageId(texture_id), .. } = data;
        self.texture_descriptors.insert(texture_id, descriptor);
    }

    pub fn load_image(&mut self, name: &str, data: Vec<u8>) {
        let (data, descriptor) = load_image_from_memory(data).unwrap();
        let image_info = self.create_image_resource(data, descriptor);
        self.images.insert(ImageSource::bundled(name), image_info);
    }

    fn render_api(&self) -> &RenderApi {
        self.render.as_ref().unwrap()
    }
}

fn load_image_from_memory(data: Vec<u8>) -> Result<(ImageData, ImageDescriptor), ImageError> {
    use image::GenericImage;
    let image = try!(image::load_from_memory(&data));
    let image_dims = image.dimensions();
    let format = match image {
        image::ImageLuma8(_) => ImageFormat::A8,
        image::ImageRgb8(_) => ImageFormat::RGB8,
        image::ImageRgba8(_) => ImageFormat::BGRA8,
        image::ImageLumaA8(_) => {
            return Err(ImageError::UnsupportedError("ImageLumaA8 unsupported".to_string()));
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

fn load_image_from_file(file: &PathBuf) -> Result<(ImageData, ImageDescriptor), ImageError> {
    use image::GenericImage;
    let image = try!(image::open(file));
    let image_dims = image.dimensions();
    let format = match image {
        image::ImageLuma8(_) => ImageFormat::A8,
        image::ImageRgb8(_) => ImageFormat::RGB8,
        image::ImageRgba8(_) => ImageFormat::BGRA8,
        image::ImageLumaA8(_) => {
            return Err(ImageError::UnsupportedError("ImageLumaA8 unsupported".to_string()));
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
        ImageFormat::RGB8 | ImageFormat::RG8 => true,
        ImageFormat::A8 => false,
        ImageFormat::Invalid | ImageFormat::RGBAF32 => unreachable!(),
    }
}

// From webrender/wrench
// These are slow. Gecko's gfx/2d/Swizzle.cpp has better versions
pub fn premultiply(data: &mut [u8]) {
    for pixel in data.chunks_mut(4) {
        let a = u32::from(pixel[3]);
        let r = u32::from(pixel[2]);
        let g = u32::from(pixel[1]);
        let b = u32::from(pixel[0]);

        pixel[3] = a as u8;
        pixel[2] = ((r * a + 128) / 255) as u8;
        pixel[1] = ((g * a + 128) / 255) as u8;
        pixel[0] = ((b * a + 128) / 255) as u8;
    }
}

