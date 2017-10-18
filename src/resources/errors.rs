error_chain! {

    foreign_links {
        ImageError(::image::ImageError);
        Io(::std::io::Error);
    }

    errors {
        EmptyFontCollection {
            description("A loaded font collection has no font attached")
            display("A loaded font collection has no font attached")
        }
        ImageFormatUnsupported(format: &'static str) {
            description("Image format is not supported")
            display("The following image format is not supported: '{}'", format)
        }
    }
}
