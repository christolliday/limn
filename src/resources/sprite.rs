//! A sprite is basically a wrapper around an image,
//! but with the difference that we don't need to re-submit the image,
//! rather we keep it around for a long time and only sample a subset of
//! the image. This is useful, for icons, for example
