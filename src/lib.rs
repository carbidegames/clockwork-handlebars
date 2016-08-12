#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate handlebars;
extern crate serde;
extern crate webapp;
extern crate clockwork;

use std::path::Path;
use clockwork::Module;
use handlebars::{Handlebars, Helper, RenderContext, Context, JsonRender};
use webapp::HtmlString;
use serde::ser::Serialize;

pub struct ViewRenderer {
    registry: Handlebars,
    layout: String,
}

impl ViewRenderer {
    pub fn new<D: AsRef<Path>, S: ToString>(directory: D, layout: S, prefix: String) -> Self {
        let mut registry = Handlebars::new();

        // Initialize the default helpers
        registry.register_helper("res", Box::new(
            move |_: &Context, h: &Helper, _: &Handlebars, rc: &mut RenderContext| {
                // We need to prepend the resource path with the prefix
                let param = h.param(0).unwrap().value().render();
                let url = format!("{}{}", prefix, param);

                try!(rc.writer.write(url.into_bytes().as_ref()));

                Ok(())
            }
        ));

        // Start at the root directory with no base
        Self::read_dir_with_base(directory, &mut registry, &"");

        ViewRenderer {
            registry: registry,
            layout: layout.to_string(),
        }
    }

    fn read_dir_with_base<D: AsRef<Path>>(directory: D, registry: &mut Handlebars, base: &str) {
        // Scan the directory for contents
        for entry in ::std::fs::read_dir(directory).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let metadata = entry.metadata().unwrap();

            // Check if we're looking at a file or a folder
            if metadata.is_file() {
                // If it's a file, add it to the registry

                // Create the full name for the template
                let template_name = path.file_stem().unwrap().to_str().unwrap();
                let template_name = format!("{}{}", base, template_name);

                // Add the template
                registry.register_template_file(&template_name, &path)
                    .map_err(|e| format!("Failed to parse {:?}: {:?}", path, e))
                    .unwrap();
            } else if metadata.is_dir() {
                // If it's a directory, recurse
                let dir_name = path.file_name().unwrap().to_str().unwrap().to_string();
                Self::read_dir_with_base(path, registry, &format!("{}{}/", base, dir_name));
            }
        }
    }

    pub fn render<M: Serialize>(&self, view: &str, model: &M) -> HtmlString {
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

#[derive(Serialize)]
struct TemplateModel {
    content: String
}
