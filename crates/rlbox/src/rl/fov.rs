//! Field of view

use crate::rl::grid2d::Vec2i;

/// Refreshes [`FovWrite`] (FoV data or maybe bundle of FoV and FoW)
pub fn refresh<T: OpacityMap>(fov: &mut impl FovWrite, params: RefreshParams<T>) {
    fov.on_refresh(&params);
    self::update_fov(fov, params.r, params.origin, params.opa);
}

/// FoV data or maybe bundle of FoV and FoW
pub trait FovWrite {
    /// Prepare for updating
    fn on_refresh<T: OpacityMap>(&mut self, params: &RefreshParams<T>);
    /// Define that the cell is in view
    ///
    /// The `pos` is guaranteed to be inside the map
    fn light(&mut self, pos: Vec2i);
}

/// Parameters to refresh FoV
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RefreshParams<'a, T: OpacityMap> {
    pub r: u32,
    pub origin: Vec2i,
    pub opa: &'a T,
}

/// Map bounds and opacities
pub trait OpacityMap {
    fn is_opaque(&self, pos: Vec2i) -> bool;
    fn contains(&self, pos: Vec2i) -> bool;
}

/// Stub implementation of [`FovWrite`]
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FovData {
    is_visible: Vec<bool>,
    radius: u32,
    /// Where the character is
    origin: Vec2i,
}

impl FovData {
    pub fn new(radius: u32, max_radius: u32) -> Self {
        let edge = max_radius * 2 + 1;
        let data = vec![false; (edge * edge) as usize];

        Self {
            is_visible: data,
            origin: Vec2i::default(),
            radius,
        }
    }

    pub fn empty() -> Self {
        Self::new(0, 0)
    }

    pub fn clear(&mut self) {
        for i in 0..self.is_visible.len() {
            self.is_visible[i] = false;
        }
    }

    pub fn radius(&self) -> u32 {
        self.radius
    }

    pub fn origin(&self) -> Vec2i {
        self.origin
    }

    fn ix(&self, pos: Vec2i) -> usize {
        let edge = self.radius * 2 + 1;
        let delta = pos - self.origin + Vec2i::new(self.radius as i32, self.radius as i32);
        (delta.x as u32 + delta.y as u32 * edge) as usize
    }

    pub fn is_in_view(&self, pos: Vec2i) -> bool {
        let delta = pos - self.origin;
        if delta.len_king() > self.radius {
            false
        } else {
            let ix = self.ix(pos);
            self.is_visible[ix]
        }
    }

    pub fn print(&self) {
        println!("r = {}", self.radius);
        for y in 0..(2 * self.radius + 1) {
            for x in 0..(2 * self.radius + 1) {
                if x == self.radius && y == self.radius {
                    print!("@");
                    continue;
                }

                let ix = x + y * (2 * self.radius + 1);
                print!(
                    "{}",
                    if self.is_visible[ix as usize] {
                        " "
                    } else {
                        "x"
                    }
                );
            }
            println!("");
        }
    }
}

impl FovWrite for FovData {
    fn on_refresh<T: OpacityMap>(&mut self, params: &RefreshParams<T>) {
        self.radius = params.r;
        self.origin = params.origin;
        // TODO: resize if needed

        self.clear();
    }

    fn light(&mut self, pos: Vec2i) {
        let ix = self.ix(pos);
        self.is_visible[ix] = true;
    }
}

// --------------------------------------------------------------------------------
// Internals

fn update_fov(fov: &mut impl FovWrite, r: u32, origin: Vec2i, opa: &impl OpacityMap) {
    fov.light(origin);
    for oct in &Octant::clockwise() {
        let mut scx = ScanContext::new(r, origin, *oct, fov, opa);
        let mut scanner = Scanner::new();
        scanner.run(1, &mut scx);
    }
}

struct ScanContext<'a, Fov: FovWrite, Opa: OpacityMap> {
    /// Radius
    r: u32,
    /// Where the character is
    origin: Vec2i,
    /// Octant
    oct: OctantContext,
    /// Field of view
    fov: &'a mut Fov,
    /// Opacity map
    opa: &'a Opa,
}

impl<'a, Fov: FovWrite, Opa: OpacityMap> ScanContext<'a, Fov, Opa> {
    pub fn new(r: u32, origin: Vec2i, oct: Octant, fov: &'a mut Fov, opa: &'a Opa) -> Self {
        Self {
            r,
            origin,
            oct: OctantContext::from_octant(oct),
            fov,
            opa,
        }
    }

    /// (row, column) -> (absolute grid position)
    pub fn rc2abs(&self, row: u32, col: u32) -> Vec2i {
        self.origin + row as i32 * self.oct.row + col as i32 * self.oct.col
    }
}

struct Scanner {
    /// Slope = col / row in range [0, 1]
    slopes: [f32; 2],
}

impl Scanner {
    fn new() -> Self {
        Self { slopes: [0.0, 1.0] }
    }

    pub fn run<T: FovWrite, U: OpacityMap>(&mut self, row_from: u32, scx: &mut ScanContext<T, U>) {
        let mut row = row_from;
        while row <= scx.r && self.scan_row(row, scx) {
            row += 1;
        }
    }

    fn col_range(&self, row: u32, r: u32) -> [u32; 2] {
        let from = self.slopes[0] * row as f32;
        let to = {
            let to = self.slopes[1] * row as f32;
            let to_max = ((r as f32 + 0.5) * (r as f32 + 0.5) - row as f32 * row as f32).sqrt();
            // FIXME: round vs floor (not tested)
            std::cmp::min(to.floor() as u32, to_max.round() as u32)
        };
        [from.ceil() as u32, to]
    }

    fn scan_row<T: FovWrite, U: OpacityMap>(
        &mut self,
        row: u32,
        scx: &mut ScanContext<T, U>,
    ) -> bool {
        if !scx.opa.contains(scx.rc2abs(row, 0)) {
            return false; // the row is out of the map
        }

        let cols = self.col_range(row, scx.r);

        if cols[1] < cols[0] {
            // the scan is too narrow to capture any cell in this row
            return false; // stop
        }

        let mut state = ScanState::Initial;

        for col in cols[0]..=cols[1] {
            let pos = scx.rc2abs(row, col);

            if !scx.opa.contains(pos) {
                // outside of map. fix the end slope to right-up coner of the cell outside of the map
                // and go to the next row if the scan it not finished.
                self.slopes[1] = (col as f32 - 0.5) / (row as f32 + 0.5);
                return state != ScanState::Opaque;
            }

            if scx.opa.is_opaque(pos) {
                if state == ScanState::Transparent {
                    let mut sub = Self {
                        // scan with end slope set to the left-up corner of the transparent cell
                        slopes: [self.slopes[0], (col as f32 - 0.5) / (row as f32 + 0.5)],
                    };
                    sub.run(row + 1, scx);
                    // start slope of _this_ scan will be updated when hitting an paque cell
                    // (if not, the octant scan will finish at the end of this procedure)
                }

                state = ScanState::Opaque;
            } else {
                if state == ScanState::Opaque {
                    // set start slope to the right-down corner of the opaque cell
                    self.slopes[0] = (col as f32 + 0.5) / (row as f32 - 0.5);

                    // consider the precision problem of diagnal-line-only FoV:
                    //
                    // #..
                    // @#.
                    if self.slopes[0] > 1.0 {
                        self.slopes[0] = 1.0;
                    }
                }

                state = ScanState::Transparent;
            }

            scx.fov.light(pos);
        }

        // permissive scan only for opaque cell
        let col = (self.slopes[1] * row as f32).ceil() as u32;
        if col > cols[1] {
            let pos = scx.rc2abs(row, col);
            if scx.opa.contains(pos) && scx.opa.is_opaque(pos) {
                scx.fov.light(pos);
                // left-up
                self.slopes[1] = (col as f32 - 0.5) / (row as f32 + 0.5);
            }
        }

        state != ScanState::Opaque
    }
}

#[derive(PartialEq)]
enum ScanState {
    /// Initial scan
    Initial,
    /// Previous scan was on opaque cell
    Opaque,
    /// Previous scan was on transparent cell
    Transparent,
}

struct OctantContext {
    row: Vec2i,
    col: Vec2i,
}

impl OctantContext {
    pub fn from_octant(oct: Octant) -> Self {
        let units = oct.to_units();
        Self {
            row: units[0],
            col: units[1],
        }
    }
}

/// Clockwise
#[derive(Debug, Clone, Copy)]
enum Octant {
    /// NEN
    A,
    /// ENE
    B,
    /// ESE
    C,
    /// SES
    D,
    E,
    F,
    G,
    H,
}

impl Octant {
    pub fn to_units(&self) -> [Vec2i; 2] {
        match self {
            Octant::A => [Vec2i::new(0, -1), Vec2i::new(1, 0)],
            Octant::B => [Vec2i::new(1, 0), Vec2i::new(0, -1)],
            Octant::C => [Vec2i::new(1, 0), Vec2i::new(0, 1)],
            Octant::D => [Vec2i::new(0, 1), Vec2i::new(1, 0)],
            Octant::E => [Vec2i::new(0, 1), Vec2i::new(-1, 0)],
            Octant::F => [Vec2i::new(-1, 0), Vec2i::new(0, 1)],
            Octant::G => [Vec2i::new(-1, 0), Vec2i::new(0, -1)],
            Octant::H => [Vec2i::new(0, -1), Vec2i::new(-1, 0)],
        }
    }

    pub const fn clockwise() -> [Self; 8] {
        use Octant::*;
        [A, B, C, D, E, F, G, H]
    }
}
