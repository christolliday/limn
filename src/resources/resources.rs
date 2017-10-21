use std::sync::{Arc, Mutex, MutexGuard};
use std::collections::HashMap;
use webrender::api::*;
use resources::font::Font;
use resources::image::Image;
use app_units::Au;
use resources::id::{Id, IdGen, WidgetId};

// In order to support removing / adding fonts, the last_font_id
// is always incremented by one if a font is added. The ID is used to generate
// a unique FontInstanceKey. Same with images.

/// Hash if a font has already been rendered to a certain size
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq)]
pub struct FontInstanceId {
    /// A key to a 
    pub font_instance_key: FontInstanceKey,
    /// The size this font was rendered as
    pub size: Au,
}

/// A fonts unique ID. Can be the postscript name when postscript support
/// lands in `rusttype`, but for the most part, this will just be the URI
/// (file name or URL) to the font, to guarantee uniqueness
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FontUID(String);

/// Global resources tracker, for fonts, widgets and images
#[derive(Debug)]
pub struct Resources {
    /// Rendering API, initialized at startup
    pub render: Option<RenderApi>,
    /// List of fonts, indexed by a unique key (u32).
    /// Note that to actually get the font, you have to look it up in `GLOBAL_FONTS`
    pub fonts: HashMap<FontInstanceId, FontUID>,
    /// List of images, indexed by a unique key
    pub images: HashMap<ImageKey, Arc<Image>>,
    /// Map of widgets and their respective IDs
    pub widget_id: IdGen<WidgetId>,
    /// The last seen font ID. Not public on purpose
    last_font_id: u64,
    /// The last assigned image ID. Not public on purpose
    last_image_id: u64,
    /// Updates needed for this frame. For performance reasons,
    /// we try to keep the amount of updates small
    /// TODO: do we have to fill this every frame or is it ok to
    /// retain resources between frames?
    current_resource_update: ResourceUpdates,
}

lazy_static! {
    /// Global, singleton-like Resources structure
    static ref GLOBAL_RESOURCES: Arc<Mutex<Resources>> = Arc::new(Mutex::new(Resources::new()));
    /// Since fonts can be re-rendered in different sizes, but without requiring to re-load the
    /// font again, the fonts are not contained in the resources.
    static ref GLOBAL_FONTS: Arc<Mutex<HashMap<FontUID, Arc<Font>>>> = Arc::new(Mutex::new(HashMap::new()));
}

/// Initialize the resources
pub(crate) fn init_resources(render_api: RenderApi) {
    GLOBAL_RESOURCES.try_lock().unwrap().render = Some(render_api);
}

// Allow global access to Resources
pub fn get_global_resources() -> MutexGuard<'static, Resources> {
    GLOBAL_RESOURCES.try_lock().unwrap()
}

/// Allow access to global fonts
pub fn get_global_fonts() -> MutexGuard<'static, HashMap<String, Arc<Font>>> {
    GLOBAL_FONTS.try_lock().unwrap()
}

/// ResourceUpdates
/// The resource updates for a given transaction (they must be applied in the same frame)
/// new
/// 
/// add_image
/// update_image
/// delete_image
///
/// add_raw_font
/// add_native_font
/// delete_font
///
/// add_font_instance
/// delete_font_instance
/// 
/// merge
/// clear

impl Resources {

    /// This function only gets called to instantiate a singleton-like resource container
    /// It should not be called ouside of limn, which is why it's not public
    pub(super) fn new() -> Self {
        Resources {
            render: None,
            fonts: HashMap::new(),
            font_instances: HashMap::new(),
            images: HashMap::new(),
            texture_descriptors: HashMap::new(),
            widget_id: IdGen::new(),
            last_font_id: 0,
            last_image_id: 0,
        }
    }

    pub fn widget_id(&mut self) -> WidgetId {
        self.widget_id.next()
    }

    // A font (all that are currently supported, TTF and OTF
    // can be uniquely identified by their "Postscript name".
    // Since this name is required to be set for a font, we
    // simply use the fonts postscript name as a unique identifer

    /// Function for checking if a font has already been loaded into the resources
    #[inline]
    pub fn get_font(&mut self, id: &FontInstanceKey) -> Option<Arc<Font>> {
        if let Some(uuid) = self.fonts.get(id) {
            if let Some(ref font) = get_global_fonts().get(uuid) {
                // clones the Arc, not the font
                font.clone()
            }
        }

        None        
    }

    /// Convenience function for getting or inserting a font
    /// This will not insert the font if it already exists (checked for equality
    /// by postscript name)
    pub fn get_font_or_insert_with<S>(&mut self, id: S, font: Font)
                                   -> Arc<Font> where S: Into<String>
    {
        if let Some(uuid) = self.fonts.get(id) {
            if let Some(ref font) = get_global_fonts().get(uuid) {
                font.clone()
            } else {
                // font and font list are out of sync, should not happen
                get_global_fonts().lock().insert(uuid, Arc::new(font));
                get_global_fonts().lock().get(id).unwrap().clone()
            }
        } else {
            get_global_fonts().lock().insert(id, Arc::new(font));
            get_global_fonts().lock().get(id).unwrap().clone()
        }
    }

    /// Inserts a font. Same as calling `get_font_or_insert_with()`.
    pub fn add_font<S>(&mut self, id: S, font: Font)
                       -> Arc<Font> where S: Into<String>
    {
        self.get_font_or_insert_with(id, font);
    }

    fn generate_font_instance_key(&mut self) -> u64 {
        // note that the order is important!
        // So you can simply say: self.fonts[font_instance_key]
        let font_instance_key = self.last_font_id;
        self.last_font_id += 1;
        font_instance_key
    }

    fn generate_image_instance_key(&mut self) -> u64 {
        let image_instance_key = self.last_image_id;
        self.last_image_id += 1;
        image_instance_key
    }

    /*
    pub fn get_font_instance(&mut self, name: &str, font_size: f32) -> &FontInstanceKey {
        let font_key = self.get_font(name).key;
        let size = Au::from_f32_px(text_layout::px_to_pt(font_size));
        if !self.font_instances.contains_key(&(name.to_owned(), size)) {
            let instance_key = self.render.as_ref().unwrap().generate_font_instance_key();
            let mut resources = ResourceUpdates::new();
            resources.add_font_instance(instance_key, font_key, size, None, None, Vec::new());
            self.render.as_ref().unwrap().update_resources(resources);
            self.font_instances.insert((name.to_owned(), size), instance_key);
        }
        &self.font_instances[&(name.to_owned(), size)]
    }*/
}
