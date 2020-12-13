//! Batcher

const N_QUADS: usize = 2048;

#[derive(Debug, Clone)]
#[repr(C)]
pub struct Vertex {
    pos: [f32; 2],
    color: [u8; 4],
    uv: [f32; 2],
}

impl<Pos, Color, Uv> From<(Pos, Color, Uv)> for Vertex
where
    Pos: Into<[f32; 2]>,
    Color: Into<[u8; 4]>,
    Uv: Into<[f32; 2]>,
{
    fn from(data: (Pos, Color, Uv)) -> Self {
        Self {
            pos: data.0.into(),
            color: data.1.into(),
            uv: data.2.into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Quad([Vertex; 4]);

/// Creates index buffer for quadliterals
///
/// Each index element has 16 bits length.
macro_rules! gen_quad_indices {
    ( $n_quads:expr ) => {{
        let mut indices = [0; 6 * $n_quads as usize];

        for q in 0..$n_quads as i16 {
            let (i, v) = (q * 6, q * 4);
            indices[i as usize] = v as i16;
            indices[(i + 1) as usize] = v + 1 as i16;
            indices[(i + 2) as usize] = v + 2 as i16;
            indices[(i + 3) as usize] = v + 3 as i16;
            indices[(i + 4) as usize] = v + 2 as i16;
            indices[(i + 5) as usize] = v + 1 as i16;
        }

        indices
    }};
}

#[derive(Debug, Clone)]
pub struct Batch {
    quads: [Quad; N_QUADS],
    ix: usize,
}

impl Batch {
    pub fn quad_mut(&mut self) -> &mut Quad {
        let ix = self.ix;
        self.ix += 1;
        &mut self.quads[ix]
    }
}

pub struct DrawCall<'a> {
    quads: &'a [Quad],
}

