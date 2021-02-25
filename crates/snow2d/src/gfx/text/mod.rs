/*!
Text rendering
*/

pub mod font;
pub mod layout;
pub mod style;

pub mod prelude {
    //! Imports all the text view types
    pub use super::{font, layout::*, style::*};
}

use rokol::fons::FonsQuad;

use crate::gfx::{batch::QuadData, draw::*, geom2d::Vec2f, Snow2d};

use self::layout::*;

/// Renders [`layout::LineLayout`]
pub fn render_line<'a>(line: &LineLayout<'a>, text: &str, base_pos: Vec2f, snow: &mut Snow2d) {
    let fb = &mut snow.fontbook;
    let batch = &mut snow.batch;

    // TODO: use quad buffer in text renderer
    // TODO: use non-immediate batcher
    let quads = fb.tex.text_iter(text).unwrap().collect::<Vec<_>>();
    for sp in &line.line_spans {
        for i in sp.quad_indices() {
            let fons_quad = &quads[i as usize];

            let q = batch.next_quad_mut(fb.tex.img());
            self::set_text_quad(q, fons_quad, base_pos, sp.style.color);
        }
    }
}

/// Sets a text quad
#[inline]
pub fn set_text_quad(q: &mut QuadData, fons_quad: &FonsQuad, base_pos: Vec2f, color: [u8; 4]) {
    // NOTE: quad = [left_up, right_up, left_down, right_down]
    q[0].uv = [fons_quad.s0, fons_quad.t0];
    q[1].uv = [fons_quad.s1, fons_quad.t0];
    q[2].uv = [fons_quad.s0, fons_quad.t1];
    q[3].uv = [fons_quad.s1, fons_quad.t1];

    q[0].pos = [
        fons_quad.x0 as f32 + base_pos.x,
        fons_quad.y0 as f32 + base_pos.y,
    ];
    q[1].pos = [
        fons_quad.x1 as f32 + base_pos.x,
        fons_quad.y0 as f32 + base_pos.y,
    ];
    q[2].pos = [
        fons_quad.x0 as f32 + base_pos.x,
        fons_quad.y1 as f32 + base_pos.y,
    ];
    q[3].pos = [
        fons_quad.x1 as f32 + base_pos.x,
        fons_quad.y1 as f32 + base_pos.y,
    ];

    q[0].color = color;
    q[1].color = color;
    q[2].color = color;
    q[3].color = color;
}

/// Sets a text quad and a shadow quad of it
#[inline]
pub fn set_text_quad_with_shadow(
    quads: &mut impl QuadIter,
    img: rokol::gfx::Image,
    fons_quad: &FonsQuad,
    base_pos: Vec2f,
    text_color: [u8; 4],
    shadow_offset: Vec2f,
    shadow_color: [u8; 4],
) {
    let q = quads.next_quad_mut(img);
    self::set_text_quad(q, fons_quad, base_pos + shadow_offset, shadow_color);
    let q = quads.next_quad_mut(img);
    self::set_text_quad(q, fons_quad, base_pos, text_color);
}
