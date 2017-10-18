use std::sync::{Mutex, MutexGuard};
use std::collections::HashMap;
use webrender::api::FontInstanceKey;

use app_units;
use text_layout;
use id::{Id, IdGen, WidgetId};
use image

pub struct Resources {
    /// Rendering API, initialized at startup
    pub render: Option<RenderApi>,
    /// List of fonts, indexed by a unique key (u32).
    pub fonts: HashMap<FontInstanceKey, Font>,
    pub images: HashMap<String, ImageInfo>,
    pub texture_descriptors: HashMap<u64, ImageDescriptor>,
    pub widget_id: IdGen<WidgetId>,
}

lazy_static! {
    /// Global, singleton-like Resources structure
    static ref RES: Mutex<Resources> = Mutex::new(Resources::new());
}

/// Initialize the resources
pub(crate) fn init_resources(render_api: RenderApi) {
    RES.try_lock().unwrap().render = Some(render_api);
}

// Allow global access to Resources
pub fn resources() -> MutexGuard<'static, Resources> {
    RES.try_lock().unwrap()
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
        }
    }
    
    pub fn widget_id(&mut self) -> WidgetId {
        self.widget_id.next()
    }

    pub fn get_font_instance(&mut self, name: &str, font_size: f32) -> &FontInstanceKey {
        let font_key = self.get_font(name).key;
        let size = app_units::Au::from_f32_px(text_layout::px_to_pt(font_size));
        if !self.font_instances.contains_key(&(name.to_owned(), size)) {
            let instance_key = self.render.as_ref().unwrap().generate_font_instance_key();
            let mut resources = ResourceUpdates::new();
            resources.add_font_instance(instance_key, font_key, size, None, None, Vec::new());
            self.render.as_ref().unwrap().update_resources(resources);
            self.font_instances.insert((name.to_owned(), size), instance_key);
        }
        &self.font_instances[&(name.to_owned(), size)]
    }
}


fn load_font_data(name: &str) -> Result<Vec<u8>, ::std::io::Error> {
    use std::fs::File;
    use std::io::Read;
    let mut file = File::open(format!("assets/fonts/{}.ttf", name)).expect("Font missing");
    let mut data = Vec::new();
    try!(file.read_to_end(&mut data));
    Ok(data)
}

pub fn load_font(name: &str) -> Result<Font, ::std::io::Error> {
    let data = try!(load_font_data(name));
    let collection = rusttype::FontCollection::from_bytes(data);
    Ok(collection.into_font().unwrap())
}
