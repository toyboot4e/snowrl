#![allow(unused)]

use {
    rand::{
        distributions::{Distribution, Uniform},
        rngs::ThreadRng,
    },
    std::{io, io::prelude::*},
};

use crate::utils::Double;

/// Cellilar automata
pub fn ca_gen_cave(size: [usize; 2], prob_init_floor: usize, n_steps: usize) -> Vec<bool> {
    let mut x = CaveGenAdvance::new(size, prob_init_floor);
    for _ in 0..n_steps {
        x.advance();
    }

    x.map.bufs.into_front()
}

pub struct CaveMap {
    /// Floor if it's true, wall if it's false. Indexed as [x + y * width]
    bufs: Double<Vec<bool>>,
    /// Width, height
    size: [usize; 2],
}

impl CaveMap {
    pub fn print(&self) -> io::Result<()> {
        let out = io::stdout();
        let mut out = out.lock();

        let cells = self.bufs.front();
        for y in 0..self.size[1] {
            for x in 0..self.size[0] {
                let ix = x + y * self.size[0];
                let c = if cells[ix] { '.' } else { '#' };
                write!(out, "{}", c)?;
            }

            writeln!(out)?;
        }

        Ok(())
    }

    fn count_neighbours(map: &[bool], size: [usize; 2], x: i32, y: i32) -> usize {
        let mut n = 0;

        for i in 0..=2 {
            for j in 0..=2 {
                let neighbour_x = x - 1 + i;
                let neighbour_y = y - 1 + j;

                if i == 1 && j == 1 {
                    continue;
                }

                // we consider edges as neighbor
                if neighbour_x < 0
                    || neighbour_y < 0
                    || neighbour_x >= size[0] as i32
                    || neighbour_y >= size[1] as i32
                    || map[neighbour_x as usize + neighbour_y as usize * size[0]]
                {
                    n += 1;
                }
            }
        }

        n
    }
}

pub struct CaveGenAdvance {
    map: CaveMap,
    rnd: ThreadRng,
}

impl CaveGenAdvance {
    pub fn new(size: [usize; 2], prob_init_floor: usize) -> Self {
        let mut rnd = rand::thread_rng();

        let mut cells = Vec::with_capacity(size[0] * size[1]);

        // fill cells with initial distribution
        let dist = Uniform::from(0..100);
        for _y in 0..size[1] {
            for _x in 0..size[0] {
                let r = dist.sample(&mut rnd);
                cells.push(r < prob_init_floor);
            }
        }

        let b = cells.clone();
        let bufs = Double::new(cells, b);

        Self {
            map: CaveMap { bufs, size },
            rnd,
        }
    }

    fn advance(&mut self) {
        let bufs = &mut self.map.bufs;

        for y in 0..self.map.size[1] {
            for x in 0..self.map.size[0] {
                let ix = x + y * self.map.size[0];

                let nbs = CaveMap::count_neighbours(bufs.back(), self.map.size, x as i32, y as i32);
                if bufs.back()[ix] {
                    // kill?
                    let death_limit = 2;
                    bufs.front_mut()[ix] = nbs > death_limit;
                } else {
                    // birth?
                    let birth_limit = 5;
                    bufs.front_mut()[ix] = nbs > birth_limit;
                }
            }
        }

        self.map.bufs.swap();
    }
}

// $ cargo test -- --nocapture --test-threads=1
// #[cfg(test)]
// mod test {
//     use super::*;

//     #[test]
//     fn test_gen() {
//         let mut gen = CaveGenAdvance::new([32, 18], 50);
//         gen.map.print().unwrap();
//         println!("-------------");
//         for _i in 0..20 {
//             gen.advance();
//         }
//         gen.map.print().unwrap();
//     }
// }
