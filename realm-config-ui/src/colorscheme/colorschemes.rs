use std::rc::Rc;

use gtk::prelude::*;
use gtk::glib;
use libcitadel::terminal::{Base16Scheme, Color};

enum RootEntry {
    Scheme(Base16Scheme),
    Category(String, Vec<Base16Scheme>),
}

impl RootEntry {
    fn key(&self) -> &str {
        match self {
            RootEntry::Scheme(scheme) => scheme.slug(),
            RootEntry::Category(name, _) => name.as_str(),
        }
    }

    fn add_to_category(list: &mut Vec<RootEntry>, category: &str, scheme: &Base16Scheme) {
        let scheme = scheme.clone();
        for entry in list.iter_mut() {
            if let RootEntry::Category(name, schemes) = entry {
                if name == category {
                    schemes.push(scheme);
                    return;
                }
            }
        }
        list.push(RootEntry::Category(category.to_string(), vec![scheme]))
    }

    fn build_list() -> Vec<RootEntry> {
        let mut list = Vec::new();
        for scheme in Base16Scheme::all_schemes() {
            if let Some(category) = scheme.category() {
                Self::add_to_category(&mut list,category, &scheme);
            } else {
                list.push(RootEntry::Scheme(scheme));
            }
        }
        list.sort_by(|a, b| a.key().cmp(b.key()));
        list
    }
}

#[derive(Clone)]
pub struct ColorSchemes {
    entries: Rc<Vec<RootEntry>>,
}

impl ColorSchemes {
    pub fn new() -> Self {
        ColorSchemes {
            entries: Rc::new(RootEntry::build_list()),
        }
    }

    pub fn populate_tree_model(&self, store: &gtk::TreeStore) {
        for entry in self.entries.iter() {
            match entry {
                RootEntry::Scheme(scheme) => {
                    let first = scheme.slug().to_string();
                    let second = scheme.name().to_string();
                    store.insert_with_values(None, None, &[(0, &first), (1, &second)]);
                }
                RootEntry::Category(name, list) => {
                    let first = String::new();
                    let second = name.to_string();
                    let iter = store.insert_with_values(None, None, &[(0, &first), (1, &second)]);
                    for scheme in list {
                        let first = scheme.slug().to_string();
                        let second = scheme.name().to_string();
                        store.insert_with_values(Some(&iter), None, &[(0, &first), (1, &second)]);
                    }
                }
            }
        }
    }

    pub fn preview_scheme(&self, id: &str) -> Option<(String, Color)> {
        let scheme = Base16Scheme::by_name(id)?;
        let bg = scheme.terminal_background();
        let text = PreviewRender::new(scheme).render_preview();
        Some((text, bg))
    }
}

struct PreviewRender {
    buffer: String,
    scheme: Base16Scheme,
}

impl PreviewRender {
    fn new(scheme: &Base16Scheme) -> Self {
        let scheme = scheme.clone();
        PreviewRender {
            buffer: String::new(),
            scheme,
        }
    }
    fn print(mut self, color_idx: usize, text: &str) -> Self {
        let s = glib::markup_escape_text(text);

        let color = self.scheme.terminal_palette_color(color_idx);
        self.color_span(Some(color), None);
        self.buffer.push_str(s.as_str());
        self.end_span();
        self
    }

    fn vtype(self, text: &str) -> Self {
        self.print(3, text)
    }

    fn konst(self, text: &str) -> Self {
        self.print(1, text)
    }

    fn func(self, text: &str) -> Self {
        self.print(4, text)
    }

    fn string(self, text: &str) -> Self {
        self.print(2, text)
    }

    fn keyword(self, text: &str) -> Self {
        self.print(5, text)
    }
    fn comment(self, text: &str) -> Self {
        self.print(8, text)
    }

    fn text(mut self, text: &str) -> Self {
        let color = self.scheme.terminal_foreground();
        self.color_span(Some(color), None);
        self.buffer.push_str(text);
        self.end_span();
        self
    }


    fn color_attrib(&mut self, name: &str, color: Color) {
        let (r,g,b) = color.rgb();
        self.buffer.push_str(&format!(" {}='#{:02X}{:02X}{:02X}'", name, r, g, b));
    }

    fn color_span(&mut self, fg: Option<Color>, bg: Option<Color>) {
        self.buffer.push_str("<span");
        if let Some(color) = fg {
            self.color_attrib("foreground", color);
        }
        if let Some(color) = bg {
            self.color_attrib("background", color);
        }
        self.buffer.push_str(">");
    }

    fn end_span(&mut self) {
        self.buffer.push_str("</span>");
    }

    fn nl(mut self) -> Self {
        self.buffer.push_str("    \n    ");
        self
    }

    fn render_colorbar(&mut self) {
        self.buffer.push_str("\n  ");
        let color = self.scheme.terminal_foreground();
        self.color_span(Some(color), None);
        for i in 0..16 {
            self.buffer.push_str(&format!(" {:X} ", i));
        }
        self.end_span();
        self.buffer.push_str("  \n  ");
        for i in 0..16 {
            let c = self.scheme.color(i);
            self.color_span(None, Some(c));
            self.buffer.push_str("   ");
            self.end_span();
        }
        self.buffer.push_str("  \n  ");
        for i in 8..16 {
            let c = self.scheme.terminal_palette_color(i);
            self.color_span(None, Some(c));
            self.buffer.push_str("      ");
            self.end_span();
        }
        self.buffer.push_str("  \n  ");
    }

    fn render_preview(mut self) -> String {
        let name = self.scheme.name().to_string();
        self.render_colorbar();
        self.nl()
            .comment("/**").nl()
            .comment(" *  An example of how this color scheme").nl()
            .comment(" *  might look in a text editor with syntax").nl()
            .comment(" *  highlighting.").nl()
            .comment(" */").nl()
            .nl()
            .func("#include ").string("<stdio.h>").nl()
            .func("#include ").string("<stdlib.h>").nl()
            .nl()
            .vtype("static char").text(" theme[] = ").string(&format!("\"{}\"", name)).text(";").nl()
            .nl()
            .vtype("int").text(" main(").vtype("int").text(" argc, ").vtype("char").text(" **argv) {").nl()
            .text("    printf(").string("\"Hello, ").keyword("%s").text("!").keyword("\\n").string("\"").text(", theme);").nl()
            .text("    exit(").konst("0").text(");").nl()
            .text("}")
            .nl()
            .nl().buffer
    }
}