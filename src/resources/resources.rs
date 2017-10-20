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

/// The fonts postscript name, used for uniquely identifying a font
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct FontPostscriptName(String);

/// Global resources tracker, for fonts, widgets and images
#[derive(Debug)]
pub struct Resources {
    /// Rendering API, initialized at startup
    pub render: Option<RenderApi>,
    /// List of fonts, indexed by a unique key (u32).
    /// Note that to actually get the font, you have to look it up in `GLOBAL_FONTS`
    pub fonts: HashMap<FontInstanceId, Arc<FontPostscriptName>>,
    /// List of images, indexed by a unique key
    pub images: HashMap<ImageKey, Arc<Image>>,
    /// Map of widgets and their respective IDs
    pub widget_id: IdGen<WidgetId>,
    /// The last seen font ID. Not public on purpose
    last_font_id: u64,
    /// The last assigned image ID. Not public on purpose
    last_image_id: u64,
}

lazy_static! {
    /// Global, singleton-like Resources structure
    static ref GLOBAL_RESOURCES: Arc<Mutex<Resources>> = Arc::new(Mutex::new(Resources::new()));
    /// Since fonts can be re-rendered in different sizes, but without requiring to re-load the
    /// font again, the fonts are not contained in the resources.
    static ref GLOBAL_FONTS: Arc<Mutex<HashMap<FontPostscriptName, Arc<Font>>>> = Arc::new(Mutex::new(HashMap::new()));
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
pub fn get_global_font_map() -> MutexGuard<'static, HashMap<String, Arc<Font>>> {
    GLOBAL_FONTS.try_lock().unwrap()
}

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
    pub fn get_font(&mut self, id: &FontInstanceKey) -> Option<Arc<FontInfo>> {
        self.fonts.get(id)
    }

    /// Convenience function for getting or inserting a font
    /// This will not insert the font if it already exists (checked for equality
    /// by postscript name)
    pub fn get_font_or_insert_with(&mut self, id: &Font) -> Arc<FontInfo> {
        
    }

    /// Inserts a font
    pub fn add_font(&mut self, font: Font) -> Arc<FontInfo> {
        
    }

    fn generate_font_instance_key(&mut self) -> u64 {
        // note that the order is important!
        // So you can simply say: self.fonts[font_instance_key]
        let font_instance_key = self.last_font_id;
        last_font_id += 1;
        font_instance_key
    }

    fn generate_image_instance_key(&mut self) -> u64 {
        let image_instance_key = self.last_image_id;
        last_image_id += 1;
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
