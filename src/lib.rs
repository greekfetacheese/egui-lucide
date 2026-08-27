//! Lucide SVG icons for [`egui`].
//!
//! Apps must register the SVG loader once:
//!
//! ```ignore
//! egui_extras::install_image_loaders(&ctx);
//! ```
//!
//! ```
//! # egui::__run_test_ui(|ui| {
//! use egui_lucide::Lucide;
//! ui.add(Lucide::House.size(18.0).stroke_width(1.75));
//! # });
//! ```
//!
//! [`LucideIcon::source`] / [`LucideIcon::image`] bake color and stroke into the SVG
//! bytes. Display size is applied on the [`egui::Image`], not in the path data.
//! The texture cache key includes color and stroke so restyling is not stale.

mod generated;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Mutex, OnceLock};

use egui::load::Bytes;
use egui::{Color32, Image, ImageSource, Response, Ui, Vec2, Widget};

pub use generated::{ICON_COUNT, LUCIDE_VERSION, Lucide};

use generated::{INNERS, NAMES};

/// Lucide defaults: 24×24 viewBox, stroke width 2.
pub const DEFAULT_SIZE: f32 = 24.0;
/// Lucide default `stroke-width`.
pub const DEFAULT_STROKE_WIDTH: f32 = 2.0;

impl Lucide {
    /// Kebab-case Lucide name, e.g. `"house"`.
    #[inline]
    pub const fn name(self) -> &'static str {
        NAMES[self as usize]
    }

    /// SVG children only (no root `<svg>`). Used when wrapping with style.
    #[inline]
    pub const fn inner(self) -> &'static str {
        INNERS[self as usize]
    }

    /// Lookup by kebab-case name (`"house"`).
    pub fn from_name(name: &str) -> Option<Self> {
        let index = NAMES.iter().position(|&n| n == name)?;
        Some(Self::ALL[index])
    }

    /// Start a styled icon. Same as [`LucideIcon::new`].
    #[inline]
    pub const fn icon(self) -> LucideIcon {
        LucideIcon::new(self)
    }

    /// Display size in points (the rasterized square).
    #[inline]
    pub const fn size(self, size: f32) -> LucideIcon {
        LucideIcon::new(self).size(size)
    }

    /// Stroke color. When omitted, [`Widget`] uses `ui.visuals().text_color()`
    /// and [`LucideIcon::source`] / [`LucideIcon::image`] use [`Color32::WHITE`].
    #[inline]
    pub const fn color(self, color: Color32) -> LucideIcon {
        LucideIcon::new(self).color(color)
    }

    /// SVG `stroke-width` in viewBox units (Lucide default is 2).
    #[inline]
    pub const fn stroke_width(self, width: f32) -> LucideIcon {
        LucideIcon::new(self).stroke_width(width)
    }

    /// [`ImageSource::Bytes`] with default size/stroke and white stroke color.
    #[inline]
    pub fn source(self) -> ImageSource<'static> {
        LucideIcon::new(self).source()
    }

    /// Ready-to-add [`Image`] with default size/stroke and white stroke color.
    #[inline]
    pub fn image(self) -> Image<'static> {
        LucideIcon::new(self).image()
    }
}

impl fmt::Display for Lucide {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl Widget for Lucide {
    fn ui(self, ui: &mut Ui) -> Response {
        LucideIcon::new(self).ui(ui)
    }
}

/// A Lucide icon with color, stroke width, and display size.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LucideIcon {
    icon: Lucide,
    size: f32,
    color: Option<Color32>,
    stroke_width: f32,
}

impl LucideIcon {
    /// Lucide defaults: size 24, stroke width 2, color unset.
    #[inline]
    pub const fn new(icon: Lucide) -> Self {
        Self {
            icon,
            size: DEFAULT_SIZE,
            color: None,
            stroke_width: DEFAULT_STROKE_WIDTH,
        }
    }

    #[inline]
    pub const fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    #[inline]
    pub const fn color(mut self, color: Color32) -> Self {
        self.color = Some(color);
        self
    }

    #[inline]
    pub const fn stroke_width(mut self, width: f32) -> Self {
        self.stroke_width = width;
        self
    }

    #[inline]
    pub const fn icon_id(self) -> Lucide {
        self.icon
    }

    fn resolved_color(self) -> Color32 {
        self.color.unwrap_or(Color32::WHITE)
    }

    /// Unique `bytes://` URI. Color and stroke are part of the key so egui
    /// does not reuse a texture from a previous style.
    ///
    /// The path **ends with `.svg`** — `egui_extras::SvgLoader` only matches
    /// that suffix, and ignores query strings.
    pub fn uri(&self) -> String {
        uri_for(self.icon, self.resolved_color(), self.stroke_width)
    }

    /// Styled SVG document (root attributes + icon inner).
    pub fn svg(&self) -> String {
        styled_svg(self.icon, self.resolved_color(), self.stroke_width)
    }

    /// [`ImageSource::Bytes`] for this style. Display size is not baked in.
    pub fn source(&self) -> ImageSource<'static> {
        let color = self.resolved_color();
        let bytes = cached_bytes(self.icon, color, self.stroke_width);
        ImageSource::Bytes {
            uri: Cow::Owned(uri_for(self.icon, color, self.stroke_width)),
            bytes,
        }
    }

    /// [`Image`] fitted to [`LucideIcon::size`].
    pub fn image(&self) -> Image<'static> {
        Image::new(self.source())
            .fit_to_exact_size(Vec2::splat(self.size))
            .alt_text(self.icon.name())
            .show_loading_spinner(false)
    }
}

impl Widget for LucideIcon {
    fn ui(self, ui: &mut Ui) -> Response {
        let color = self.color.unwrap_or_else(|| ui.visuals().text_color());
        self.color(color).image().ui(ui)
    }
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
struct CacheKey {
    id: u16,
    rgba: u32,
    stroke_bits: u32,
}

fn color_rgba(color: Color32) -> u32 {
    u32::from_be_bytes([color.r(), color.g(), color.b(), color.a()])
}

fn cache() -> &'static Mutex<HashMap<CacheKey, Bytes>> {
    static CACHE: OnceLock<Mutex<HashMap<CacheKey, Bytes>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

fn cached_bytes(icon: Lucide, color: Color32, stroke_width: f32) -> Bytes {
    let key = CacheKey {
        id: icon as u16,
        rgba: color_rgba(color),
        stroke_bits: stroke_width.to_bits(),
    };
    if let Ok(guard) = cache().lock() {
        if let Some(bytes) = guard.get(&key) {
            return bytes.clone();
        }
    }
    let bytes = Bytes::from(styled_svg(icon, color, stroke_width).into_bytes());
    if let Ok(mut guard) = cache().lock() {
        guard.entry(key).or_insert_with(|| bytes.clone());
    }
    bytes
}

fn uri_for(icon: Lucide, color: Color32, stroke_width: f32) -> String {
    // SvgLoader matches `uri.ends_with(".svg")` only — no query string.
    format!(
        "bytes://lucide/{}.c={:08x}.sw={:08x}.svg",
        icon.name(),
        color_rgba(color),
        stroke_width.to_bits()
    )
}

fn svg_color(color: Color32) -> String {
    if color.a() == 255 {
        format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b())
    } else {
        format!(
            "rgba({},{},{},{:.4})",
            color.r(),
            color.g(),
            color.b(),
            f32::from(color.a()) / 255.0
        )
    }
}

fn styled_svg(icon: Lucide, color: Color32, stroke_width: f32) -> String {
    let inner = icon.inner();
    let color = svg_color(color);
    let mut out = String::with_capacity(inner.len() + 192);
    out.push_str(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"24\" height=\"24\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"",
    );
    out.push_str(&color);
    out.push_str("\" stroke-width=\"");
    out.push_str(&format_stroke(stroke_width));
    out.push_str("\" stroke-linecap=\"round\" stroke-linejoin=\"round\">");
    out.push_str(inner);
    out.push_str("</svg>");
    out
}

fn format_stroke(width: f32) -> String {
    let rounded = (width * 1000.0).round() / 1000.0;
    if (rounded - rounded.round()).abs() < f32::EPSILON {
        format!("{}", rounded as i32)
    } else {
        format!("{rounded}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_match() {
        assert_eq!(Lucide::ALL.len(), ICON_COUNT);
        assert_eq!(NAMES.len(), ICON_COUNT);
        assert_eq!(INNERS.len(), ICON_COUNT);
        assert!(!LUCIDE_VERSION.is_empty());
    }

    #[test]
    fn house_roundtrip() {
        let icon = Lucide::from_name("house").expect("house");
        assert_eq!(icon, Lucide::House);
        assert_eq!(icon.name(), "house");
        assert_eq!(icon.to_string(), "house");
        assert!(icon.inner().contains("<path"));
    }

    #[test]
    fn names_are_unique_and_nonempty() {
        let mut seen = std::collections::HashSet::new();
        for icon in Lucide::ALL {
            assert!(!icon.name().is_empty());
            assert!(!icon.inner().is_empty());
            assert!(seen.insert(icon.name()));
            assert_eq!(icon as usize, seen.len() - 1);
        }
    }

    #[test]
    fn svg_bakes_color_and_stroke() {
        let icon = Lucide::House
            .color(Color32::from_rgb(0x12, 0x34, 0x56))
            .stroke_width(1.75);
        let svg = icon.svg();
        assert!(svg.contains("stroke=\"#123456\""));
        assert!(svg.contains("stroke-width=\"1.75\""));
        assert!(svg.contains(Lucide::House.inner()));
        assert!(icon.uri().contains("house."));
        assert!(icon.uri().ends_with(".svg"));
        assert_ne!(
            icon.uri(),
            Lucide::House.color(Color32::WHITE).uri()
        );
        assert_ne!(
            icon.uri(),
            Lucide::House
                .color(Color32::from_rgb(0x12, 0x34, 0x56))
                .stroke_width(2.0)
                .uri()
        );
    }

    #[test]
    fn source_is_bytes_with_svg_suffix() {
        let source = Lucide::House.source();
        match source {
            ImageSource::Bytes { uri, bytes } => {
                assert!(uri.ends_with(".svg"));
                assert!(uri.starts_with("bytes://lucide/"));
                assert!(!bytes.as_ref().is_empty());
            }
            other => panic!("expected Bytes, got {other:?}"),
        }
    }
}
