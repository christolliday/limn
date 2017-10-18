error_chain! {
    extern_links {
        ImageError(::image::ImageError),
        Io(::std::io::Error),
    }
}
