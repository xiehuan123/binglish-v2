use ab_glyph::{Font, FontArc, GlyphId, PxScale, ScaleFont};
use image::imageops::FilterType;
use image::{DynamicImage, Rgba, RgbaImage};
use imageproc::drawing::draw_text_mut;
use std::path::Path;
use std::sync::OnceLock;

const FONT_EN: &[u8] = include_bytes!("../resources/Lato-Bold.ttf");
const FONT_CN: &[u8] = include_bytes!("../resources/SimHei.ttf");

const WHITE: Rgba<u8> = Rgba([255, 255, 255, 255]);
const SHADOW: Rgba<u8> = Rgba([0, 0, 0, 120]);

pub struct WordCard {
    pub word: String,
    pub desc: Option<String>,
    pub sentence_en: Option<String>,
    pub sentence_cn: Option<String>,
}

struct DualFont {
    en: FontArc,
    cn: FontArc,
}

static FONTS: OnceLock<DualFont> = OnceLock::new();

fn get_fonts() -> &'static DualFont {
    FONTS.get_or_init(|| DualFont {
        en: FontArc::try_from_slice(FONT_EN).expect("Failed to load EN font"),
        cn: FontArc::try_from_slice(FONT_CN).expect("Failed to load CN font"),
    })
}

impl DualFont {
    fn pick(&self, ch: char) -> &FontArc {
        if ch.is_ascii() || self.en.glyph_id(ch) != GlyphId(0) {
            &self.en
        } else {
            &self.cn
        }
    }

    fn char_advance(&self, ch: char, scale: PxScale) -> f32 {
        let font = self.pick(ch);
        font.as_scaled(scale).h_advance(font.glyph_id(ch))
    }

    fn line_height(&self, scale: PxScale) -> f32 {
        let en_h = self.en.as_scaled(scale).ascent() - self.en.as_scaled(scale).descent();
        let cn_h = self.cn.as_scaled(scale).ascent() - self.cn.as_scaled(scale).descent();
        en_h.max(cn_h)
    }
}

fn split_by_font<'a>(fonts: &'a DualFont, text: &str) -> Vec<(&'a FontArc, String)> {
    let mut segments: Vec<(&FontArc, String)> = Vec::new();
    for ch in text.chars() {
        let font = fonts.pick(ch);
        let font_ptr = font as *const FontArc;
        if let Some(last) = segments.last_mut() {
            if std::ptr::eq(last.0, font_ptr) {
                last.1.push(ch);
                continue;
            }
        }
        segments.push((font, ch.to_string()));
    }
    segments
}

fn draw_mixed_text(
    canvas: &mut RgbaImage,
    fonts: &DualFont,
    scale: PxScale,
    text: &str,
    mut x: i32,
    y: i32,
    color: Rgba<u8>,
) {
    for (font, seg) in split_by_font(fonts, text) {
        draw_text_mut(canvas, color, x, y, scale, font, &seg);
        let scaled = font.as_scaled(scale);
        x += seg.chars().map(|c| scaled.h_advance(font.glyph_id(c))).sum::<f32>() as i32;
    }
}

fn wrap_text(fonts: &DualFont, scale: PxScale, text: &str, max_w: f32, out: &mut Vec<String>) {
    let mut line = String::new();
    let mut line_w: f32 = 0.0;
    for ch in text.chars() {
        let advance = fonts.char_advance(ch, scale);
        if line_w + advance > max_w && !line.is_empty() {
            out.push(std::mem::take(&mut line));
            line_w = 0.0;
        }
        line.push(ch);
        line_w += advance;
    }
    if !line.is_empty() {
        out.push(line);
    }
}

fn draw_overlay(canvas: &mut RgbaImage, y_start: u32, height: u32, width: u32) {
    let alpha = 100.0 / 255.0;
    let inv = 1.0 - alpha;
    for py in y_start..y_start + height {
        for px in 0..width {
            let p = canvas.get_pixel_mut(px, py);
            p.0[0] = (p.0[0] as f32 * inv) as u8;
            p.0[1] = (p.0[1] as f32 * inv) as u8;
            p.0[2] = (p.0[2] as f32 * inv) as u8;
        }
    }
}

pub fn render_word_on_image(
    base_image_path: &Path,
    card: &WordCard,
    output_path: &Path,
    screen_width: u32,
    screen_height: u32,
) -> Result<(), String> {
    let img = image::open(base_image_path)
        .map_err(|e| format!("Failed to open base image: {e}"))?;
    let mut canvas = resize_cover(img, screen_width, screen_height);

    let fonts = get_fonts();
    let h = screen_height as f32;
    let w = screen_width as f32;

    let text_scale = PxScale::from(h / 25.0);
    let line_h = fonts.line_height(text_scale);
    let line_gap = h / 60.0;
    let padding_x = (w * 0.04) as i32;
    let padding_y = h * 0.025;
    let shadow_off = (h / 500.0).max(1.0) as i32;
    let max_text_w = w - padding_x as f32 * 2.0;

    let mut lines: Vec<String> = Vec::new();
    let first = match &card.desc {
        Some(desc) => format!("{} {}", card.word, desc),
        None => card.word.clone(),
    };
    wrap_text(fonts, text_scale, &first, max_text_w, &mut lines);
    if let Some(ref en) = card.sentence_en {
        wrap_text(fonts, text_scale, en, max_text_w, &mut lines);
    }
    if let Some(ref cn) = card.sentence_cn {
        wrap_text(fonts, text_scale, cn, max_text_w, &mut lines);
    }

    let num_lines = lines.len() as f32;
    let overlay_h = (num_lines * line_h + (num_lines - 1.0) * line_gap + padding_y * 2.0) as u32;
    let overlay_y = screen_height.saturating_sub(overlay_h + (h * 0.04) as u32);

    draw_overlay(&mut canvas, overlay_y, overlay_h, screen_width);

    let mut y = overlay_y as i32 + padding_y as i32;
    for line in &lines {
        draw_mixed_text(&mut canvas, fonts, text_scale, line, padding_x + shadow_off, y + shadow_off, SHADOW);
        draw_mixed_text(&mut canvas, fonts, text_scale, line, padding_x, y, WHITE);
        y += line_h as i32 + line_gap as i32;
    }

    DynamicImage::ImageRgba8(canvas)
        .to_rgb8()
        .save_with_format(output_path, image::ImageFormat::Jpeg)
        .map_err(|e| format!("Failed to save wallpaper: {e}"))?;

    Ok(())
}

fn resize_cover(img: DynamicImage, target_w: u32, target_h: u32) -> RgbaImage {
    let (src_w, src_h) = (img.width(), img.height());
    let scale = (target_w as f64 / src_w as f64).max(target_h as f64 / src_h as f64);
    let new_w = (src_w as f64 * scale).ceil() as u32;
    let new_h = (src_h as f64 * scale).ceil() as u32;
    let resized = img.resize_exact(new_w, new_h, FilterType::Triangle);
    let crop_x = (new_w.saturating_sub(target_w)) / 2;
    let crop_y = (new_h.saturating_sub(target_h)) / 2;
    image::imageops::crop_imm(&resized.to_rgba8(), crop_x, crop_y, target_w, target_h).to_image()
}
