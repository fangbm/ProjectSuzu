use cosmic_text::{
    Attrs, Buffer, Color as TextColor, FontSystem, Metrics, Shaping, SwashCache, Wrap,
};
use suzu_core::Color;

use super::{
    frame::{FrameText, FrameTexture},
    texture::channel,
};

pub(super) fn rasterize_text(
    texture_id: &str,
    text: &FrameText,
    font_system: &mut FontSystem,
    swash_cache: &mut SwashCache,
) -> FrameTexture {
    let width = text.bounds.size.x.ceil().max(1.0) as u32;
    let height = text.bounds.size.y.ceil().max(1.0) as u32;
    let mut rgba = vec![0; width as usize * height as usize * 4];
    let font_size = text_font_size(text.bounds);
    let mut buffer = Buffer::new(font_system, Metrics::new(font_size, font_size * 1.35));
    buffer.set_size(font_system, Some(width as f32), Some(height as f32));
    buffer.set_wrap(font_system, Wrap::WordOrGlyph);
    buffer.set_text(font_system, &text.content, &Attrs::new(), Shaping::Advanced);
    buffer.draw(
        font_system,
        swash_cache,
        text_color(text.color),
        |x, y, _w, _h, color| {
            if x < 0 || y < 0 {
                return;
            }
            let x = x as u32;
            let y = y as u32;
            if x >= width || y >= height {
                return;
            }
            blend_pixel(&mut rgba, width, x, y, color.as_rgba());
        },
    );

    FrameTexture::new(texture_id, width, height, rgba)
}

fn text_font_size(bounds: suzu_core::Rect) -> f32 {
    bounds.size.y.mul_add(0.28, 0.0).clamp(18.0, 30.0)
}

fn text_color(color: Color) -> TextColor {
    TextColor::rgba(
        channel(color.r),
        channel(color.g),
        channel(color.b),
        channel(color.a),
    )
}

fn blend_pixel(rgba: &mut [u8], width: u32, x: u32, y: u32, source: [u8; 4]) {
    let offset = ((y * width + x) * 4) as usize;
    let src_a = source[3] as f32 / 255.0;
    if src_a <= 0.0 {
        return;
    }

    let dst = [
        rgba[offset],
        rgba[offset + 1],
        rgba[offset + 2],
        rgba[offset + 3],
    ];
    let dst_a = dst[3] as f32 / 255.0;
    let out_a = src_a + dst_a * (1.0 - src_a);
    if out_a <= 0.0 {
        return;
    }

    for channel_index in 0..3 {
        let src = source[channel_index] as f32 / 255.0;
        let dst = dst[channel_index] as f32 / 255.0;
        let out = (src * src_a + dst * dst_a * (1.0 - src_a)) / out_a;
        rgba[offset + channel_index] = channel(out);
    }
    rgba[offset + 3] = channel(out_a);
}
