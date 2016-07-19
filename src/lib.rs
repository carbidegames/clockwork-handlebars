extern crate clockwork;
extern crate handlebars;
extern crate rustc_serialize;
extern crate webutil;

use std::path::Path;
use std::collections::BTreeMap;
use clockwork::Module;
use handlebars::Handlebars;
use rustc_serialize::json::{Json, ToJson};
use webutil::HtmlString;

pub struct ViewRenderer {
    registry: Handlebars,
    layout: String,
}

impl ViewRenderer {
    pub fn new<D: AsRef<Path>, S: ToString>(directory: D, layout: S) -> Self {
        let mut registry = Handlebars::new();

        // Scan the views directory for files
        for entry in ::std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let metadata = entry.metadata().unwrap();

            // Skip non-files
            if !metadata.is_file() {
                continue;
            }

            // Add the template
            let template_name = path.file_stem().unwrap();
            registry.register_template_file(template_name.to_str().unwrap(), &path)
                .map_err(|e| format!("Failed to parse {:?}: {:?}", path, e))
                .unwrap();
        }

        ViewRenderer {
            registry: registry,
            layout: layout.to_string(),
        }
    }

    pub fn render<M: ToJson>(&self, view: &str, model: &M) -> HtmlString {
        // Render the specific view
        let content = self.registry.render(view, model).unwrap();

        // Render the vew into the layout
        let template_model = TemplateModel {content: content};
        let html = self.registry.render(&self.layout, &template_model).unwrap();

        // Bless the result and pass it on
        HtmlString::bless(html)
    }
}

impl Module for ViewRenderer {
}

struct TemplateModel {
    content: String
}

impl ToJson for TemplateModel {
    fn to_json(&self) -> Json {
        let mut m: BTreeMap<String, Json> = BTreeMap::new();
        m.insert("content".into(), self.content.to_json());
        m.to_json()
    }
}
