use std::collections::HashSet;
use std::convert::TryFrom;

use gtk::{gdk,gio};
use gtk::gio::prelude::*;
use rand::Rng;
use libcitadel::Realm;

pub struct CitadelSettings {
    settings: gio::Settings,
    frame_colors: Vec<gdk::RGBA>,
    realms: Vec<RealmFrameColor>,
    used_colors: HashSet<gdk::RGBA>,
}

#[derive(Clone)]
struct RealmFrameColor(String,gdk::RGBA);

impl RealmFrameColor {

    fn new(realm: &str, color: &gdk::RGBA) -> Self {
        RealmFrameColor(realm.to_string(), color.clone())
    }

    fn realm(&self) -> &str {
        &self.0
    }

    fn color(&self) -> &gdk::RGBA {
        &self.1
    }

    fn set_color(&mut self, color: gdk::RGBA) {
        self.1 = color;
    }
}

impl CitadelSettings {

    fn choose_random_color(&self) -> gdk::RGBA {
        if !self.frame_colors.is_empty() {
            let n = rand::thread_rng().gen_range(0..self.frame_colors.len());
            self.frame_colors[n].clone()
        } else {
            gdk::RGBA::blue()
        }
    }

    fn allocate_color(&self) -> gdk::RGBA {
        self.frame_colors.iter()
            .find(|&c| !self.used_colors.contains(c))
            .cloned()
            .unwrap_or_else(|| self.choose_random_color())
    }

    pub fn get_realm_color(&self, name: Option<&str>) -> gdk::RGBA {
        name.and_then(|name| self.get_realm_frame_color(name))
            .cloned()
            .unwrap_or_else(|| self.allocate_color())
    }

    pub fn store_realm_color(&mut self, name: &str, color: gdk::RGBA) -> bool {
        if let Some(realm) = self.realms.iter_mut().find(|r| r.realm() == name) {
            realm.set_color(color);
        } else {
            self.realms.push(RealmFrameColor::new(name, &color));
        }

        let list = self.realms.iter().map(|r| r.to_string()).collect::<Vec<String>>();
        let realms = list.iter().map(|s| s.as_str()).collect::<Vec<&str>>();
        self.settings.set_strv("realm-frame-colors", &realms).is_ok()
    }

    pub fn new() -> Self {
        let settings = gio::Settings::new("com.subgraph.citadel");

        let realms = settings.strv("realm-frame-colors")
            .into_iter()
            .flat_map(|gs| RealmFrameColor::try_from(gs.as_str()).ok())
            .collect::<Vec<RealmFrameColor>>();

        let frame_colors = settings.strv("frame-color-list").into_iter()
            .flat_map(|gs| gs.as_str().parse().ok())
            .collect();

        let used_colors = realms.iter()
            .map(|rfc| rfc.1.clone()).collect();

        CitadelSettings {
            settings,
            frame_colors,
            realms,
            used_colors,
        }
    }

    fn get_realm_frame_color(&self, name: &str) -> Option<&gdk::RGBA> {
        self.realms.iter()
            .find(|r| r.realm() == name)
            .map(|r| r.color())
    }
}

impl TryFrom<&str> for RealmFrameColor {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let idx = value.find(':').ok_or(())?;
        let (realm, color_str) = value.split_at(idx);

        let rgba = &color_str[1..].parse::<gdk::RGBA>()
            .map_err(|_| ())?;

        if Realm::is_valid_name(realm) {
            Ok(RealmFrameColor::new(realm, rgba))
        } else {
            Err(())
        }
    }
}

impl ToString for RealmFrameColor {
    fn to_string(&self) -> String {
        format!("{}:{}", self.realm(), self.color())
    }
}
