use std::sync::{Arc, RwLock};

use js_sys::ArrayBuffer;
use typst::{
    font::{Font, FontBook, FontFlags, FontInfo, FontStretch, FontStyle, FontVariant, FontWeight},
    util::Buffer,
};
use typst_ts_core::{
    cache::FontInfoCache,
    font::{BufferFontLoader, FontProfile, FontResolverImpl, PartialFontBook},
    FontLoader, FontSlot,
};
use wasm_bindgen::prelude::*;

use crate::font::info::typst_typographic_family;

/// Destructures a JS `[key, value]` pair into a tuple of [`Deserializer`]s.
pub(crate) fn convert_pair(pair: JsValue) -> (JsValue, JsValue) {
    let pair = pair.unchecked_into::<js_sys::Array>();
    (pair.get(0), pair.get(1))
}
struct FontBuilder {}

fn font_family_web_to_typst(family: &str, full_name: &str) -> Result<String, JsValue> {
    let mut family = family;
    if family.starts_with("Noto")
        || family.starts_with("NewCM")
        || family.starts_with("NewComputerModern")
    {
        family = full_name;
    }

    if family.is_empty() {
        return Err(JsValue::from_str(
            "empty family (cannot infer from font.family and font.fullName)",
        ));
    }

    Ok(typst_typographic_family(family).to_string())
}

struct WebFontInfo {
    family: String,
    full_name: String,
    postscript_name: String,
    style: String,
}

fn infer_info_from_web_font(
    WebFontInfo {
        family,
        full_name,
        postscript_name,
        style,
    }: WebFontInfo,
) -> Result<FontInfo, JsValue> {
    let family = font_family_web_to_typst(&family, &full_name)?;

    let mut full = full_name;
    full.make_ascii_lowercase();

    let mut postscript = postscript_name;
    postscript.make_ascii_lowercase();

    let mut style = style;
    style.make_ascii_lowercase();

    let search_scopes = [style.as_str(), postscript.as_str(), full.as_str()];

    let variant = {
        // Some fonts miss the relevant bits for italic or oblique, so
        // we also try to infer that from the full name.
        let italic = full.contains("italic");
        let oblique = full.contains("oblique") || full.contains("slanted");

        let style = match (italic, oblique) {
            (false, false) => FontStyle::Normal,
            (true, _) => FontStyle::Italic,
            (_, true) => FontStyle::Oblique,
        };

        let weight = {
            let mut weight = None;
            let mut secondary_weight = None;
            'searchLoop: for &search_style in &[
                "thin",
                "extralight",
                "extra light",
                "extra-light",
                "light",
                "regular",
                "medium",
                "semibold",
                "semi bold",
                "semi-bold",
                "bold",
                "extrabold",
                "extra bold",
                "extra-bold",
                "black",
            ] {
                for (idx, &search_scope) in search_scopes.iter().enumerate() {
                    if search_scope.contains(search_style) {
                        let guess_weight = match search_style {
                            "thin" => Some(FontWeight::THIN),
                            "extralight" => Some(FontWeight::EXTRALIGHT),
                            "extra light" => Some(FontWeight::EXTRALIGHT),
                            "extra-light" => Some(FontWeight::EXTRALIGHT),
                            "light" => Some(FontWeight::LIGHT),
                            "regular" => Some(FontWeight::REGULAR),
                            "medium" => Some(FontWeight::MEDIUM),
                            "semibold" => Some(FontWeight::SEMIBOLD),
                            "semi bold" => Some(FontWeight::SEMIBOLD),
                            "semi-bold" => Some(FontWeight::SEMIBOLD),
                            "bold" => Some(FontWeight::BOLD),
                            "extrabold" => Some(FontWeight::EXTRABOLD),
                            "extra bold" => Some(FontWeight::EXTRABOLD),
                            "extra-bold" => Some(FontWeight::EXTRABOLD),
                            "black" => Some(FontWeight::BLACK),
                            _ => unreachable!(),
                        };

                        if let Some(guess_weight) = guess_weight {
                            if idx == 0 {
                                weight = Some(guess_weight);
                                break 'searchLoop;
                            } else {
                                secondary_weight = Some(guess_weight);
                            }
                        }
                    }
                }
            }

            weight.unwrap_or(secondary_weight.unwrap_or(FontWeight::REGULAR))
        };

        let stretch = {
            let mut stretch = None;
            'searchLoop: for &search_style in &[
                "ultracondensed",
                "ultra_condensed",
                "ultra-condensed",
                "extracondensed",
                "extra_condensed",
                "extra-condensed",
                "condensed",
                "semicondensed",
                "semi_condensed",
                "semi-condensed",
                "normal",
                "semiexpanded",
                "semi_expanded",
                "semi-expanded",
                "expanded",
                "extraexpanded",
                "extra_expanded",
                "extra-expanded",
                "ultraexpanded",
                "ultra_expanded",
                "ultra-expanded",
            ] {
                for (idx, &search_scope) in search_scopes.iter().enumerate() {
                    if search_scope.contains(search_style) {
                        let guess_stretch = match search_style {
                            "ultracondensed" => Some(FontStretch::ULTRA_CONDENSED),
                            "ultra_condensed" => Some(FontStretch::ULTRA_CONDENSED),
                            "ultra-condensed" => Some(FontStretch::ULTRA_CONDENSED),
                            "extracondensed" => Some(FontStretch::EXTRA_CONDENSED),
                            "extra_condensed" => Some(FontStretch::EXTRA_CONDENSED),
                            "extra-condensed" => Some(FontStretch::EXTRA_CONDENSED),
                            "condensed" => Some(FontStretch::CONDENSED),
                            "semicondensed" => Some(FontStretch::SEMI_CONDENSED),
                            "semi_condensed" => Some(FontStretch::SEMI_CONDENSED),
                            "semi-condensed" => Some(FontStretch::SEMI_CONDENSED),
                            "normal" => Some(FontStretch::NORMAL),
                            "semiexpanded" => Some(FontStretch::SEMI_EXPANDED),
                            "semi_expanded" => Some(FontStretch::SEMI_EXPANDED),
                            "semi-expanded" => Some(FontStretch::SEMI_EXPANDED),
                            "expanded" => Some(FontStretch::EXPANDED),
                            "extraexpanded" => Some(FontStretch::EXTRA_EXPANDED),
                            "extra_expanded" => Some(FontStretch::EXTRA_EXPANDED),
                            "extra-expanded" => Some(FontStretch::EXTRA_EXPANDED),
                            "ultraexpanded" => Some(FontStretch::ULTRA_EXPANDED),
                            "ultra_expanded" => Some(FontStretch::ULTRA_EXPANDED),
                            "ultra-expanded" => Some(FontStretch::ULTRA_EXPANDED),
                            _ => None,
                        };

                        if let Some(guess_stretch) = guess_stretch {
                            if idx == 0 {
                                stretch = Some(guess_stretch);
                                break 'searchLoop;
                            }
                        }
                    }
                }
            }

            stretch.unwrap_or(FontStretch::NORMAL)
        };

        FontVariant {
            style,
            weight,
            stretch,
        }
    };

    let flags = {
        // guess mono and serif
        let mut flags = FontFlags::empty();

        for search_scope in search_scopes {
            if search_scope.contains("mono") {
                flags |= FontFlags::MONOSPACE;
            } else if search_scope.contains("serif") {
                flags |= FontFlags::SERIF;
            }
        }

        flags
    };
    let coverage = serde_json::from_str("[0, 4294967295]").unwrap();

    Ok(FontInfo {
        family,
        variant,
        flags,
        coverage,
    })
}

impl FontBuilder {
    // fn to_f64(&self, field: &str, val: &JsValue) -> Result<f64, JsValue> {
    //     Ok(val
    //         .as_f64()
    //         .ok_or_else(|| JsValue::from_str(&format!("expected f64 for {}, got {:?}", field, val)))
    //         .unwrap())
    // }

    fn to_string(&self, field: &str, val: &JsValue) -> Result<String, JsValue> {
        Ok(val
            .as_string()
            .ok_or_else(|| {
                JsValue::from_str(&format!("expected string for {}, got {:?}", field, val))
            })
            .unwrap())
    }

    fn font_web_to_typst(
        &self,
        val: &JsValue,
    ) -> Result<(JsValue, js_sys::Function, Vec<typst::font::FontInfo>), JsValue> {
        let mut postscript_name = String::new();
        let mut family = String::new();
        let mut full_name = String::new();
        let mut style = String::new();
        let mut font_ref = None;
        let mut font_blob_loader = None;
        let mut font_cache: Option<FontInfoCache> = None;

        for (k, v) in js_sys::Object::entries(val.dyn_ref().ok_or_else(|| {
            JsValue::from_str(&format!(
                "expected object for iterating font, got {:?}",
                val
            ))
        })?)
        .iter()
        .map(convert_pair)
        {
            let k = self.to_string("web_font.key", &k)?;
            match k.as_str() {
                "postscriptName" => {
                    postscript_name = self.to_string("web_font.postscriptName", &v)?;
                }
                "family" => {
                    family = self.to_string("web_font.family", &v)?;
                }
                "fullName" => {
                    full_name = self.to_string("web_font.fullName", &v)?;
                }
                "style" => {
                    style = self.to_string("web_font.style", &v)?;
                }
                "ref" => {
                    font_ref = Some(v);
                }
                "info" => {
                    // a previous calculated font info
                    font_cache = serde_wasm_bindgen::from_value(v).ok();
                }
                "blob" => {
                    font_blob_loader = Some(v.dyn_into()?);
                }
                _ => panic!("unknown key for {}: {}", "web_font", k),
            }
        }

        let font_info = match font_cache {
            Some(font_cache) => Some(
                // todo cache invalidatio: font_cache.conditions.iter()
                font_cache.info,
            ),
            None => None,
        };

        let font_info: Vec<FontInfo> = match font_info {
            Some(font_info) => font_info,
            None => {
                vec![infer_info_from_web_font(WebFontInfo {
                    family: family.clone(),
                    full_name,
                    postscript_name,
                    style,
                })?]
            }
        };

        Ok((
            font_ref.ok_or_else(|| {
                JsValue::from_str(&format!("Could not find font reference for {}", family,))
            })?,
            font_blob_loader.ok_or_else(|| {
                JsValue::from_str(&format!("Could not find font blob loader for {}", family,))
            })?,
            font_info,
        ))
    }
}

#[derive(Clone, Debug)]
pub struct WebFont {
    pub info: FontInfo,
    pub context: JsValue,
    pub blob: js_sys::Function,
    pub index: u32,
}

impl WebFont {
    pub fn load(&self) -> Option<ArrayBuffer> {
        self.blob
            .call1(&self.context, &self.index.into())
            .unwrap()
            .dyn_into::<ArrayBuffer>()
            .ok()
    }
}

pub struct WebFontLoader {
    font: WebFont,
    index: u32,
}

impl WebFontLoader {
    pub fn new(font: WebFont, index: u32) -> Self {
        Self { font, index }
    }
}

impl FontLoader for WebFontLoader {
    fn load(&mut self) -> Option<Font> {
        let font = &self.font;
        web_sys::console::log_3(
            &"dyn init".into(),
            &font.context,
            &format!("{:?}", font.info).into(),
        );
        // let blob = pollster::block_on(JsFuture::from(blob.array_buffer())).unwrap();
        let blob = font.load()?;
        let blob = Buffer::from(js_sys::Uint8Array::new(&blob).to_vec());

        Font::new(blob, self.index)
    }
}

/// Searches for fonts.
pub struct BrowserFontSearcher {
    pub book: FontBook,
    pub fonts: Vec<FontSlot>,
    pub profile: FontProfile,
    pub partial_book: Arc<RwLock<PartialFontBook>>,
}

impl BrowserFontSearcher {
    /// Create a new, empty system searcher.
    pub fn new() -> Self {
        let profile = FontProfile {
            version: "v1beta".to_owned(),
            ..Default::default()
        };
        Self {
            book: FontBook::new(),
            fonts: vec![],
            profile,
            partial_book: Arc::new(RwLock::new(PartialFontBook::default())),
        }
    }

    pub async fn add_web_fonts(&mut self, fonts: js_sys::Array) -> Result<(), JsValue> {
        let font_builder = FontBuilder {};

        for v in fonts.iter() {
            let (font_ref, font_blob_loader, font_info) = font_builder.font_web_to_typst(&v)?;

            for (i, info) in font_info.into_iter().enumerate() {
                self.book.push(info.clone());

                let index = self.fonts.len();
                self.fonts.push(FontSlot::new(Box::new(WebFontLoader {
                    font: WebFont {
                        info,
                        context: font_ref.clone(),
                        blob: font_blob_loader.clone(),
                        index: index as u32,
                    },
                    index: i as u32,
                })))
            }
        }

        Ok(())
    }

    pub fn add_font_data(&mut self, buffer: Buffer) {
        for (i, info) in FontInfo::iter(buffer.as_slice()).enumerate() {
            self.book.push(info);

            let buffer = buffer.clone();
            self.fonts.push(FontSlot::new(Box::new(BufferFontLoader {
                buffer: Some(buffer),
                index: i as u32,
            })))
        }
    }
}

impl Default for BrowserFontSearcher {
    fn default() -> Self {
        Self::new()
    }
}

impl From<BrowserFontSearcher> for FontResolverImpl {
    fn from(value: BrowserFontSearcher) -> Self {
        FontResolverImpl::new(value.book, value.partial_book, value.fonts, value.profile)
    }
}