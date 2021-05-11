/*!
Scenes
*/

use snow2d::{
    audio::src::Wav,
    ui::{
        anim::AnimImpl,
        anim_builder::{AnimGen, AnimInsertLog},
        node::Draw,
        Anim, AnimStorage, Layer, Node,
    },
};

use crate::utils::asset_defs::{title, AssetDef};

use crate::prelude::*;

/// Return value of title screen
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Choice {
    NewGame,
    Continue,
    Exit,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Phased<T> {
    pub selected: T,
    pub not_selected: T,
}

impl<T: Clone> Phased<T> {
    fn get(&self, is_selected: bool) -> T {
        if is_selected {
            self.selected.clone()
        } else {
            self.not_selected.clone()
        }
    }
}

impl Choice {
    pub fn from_usize(x: usize) -> Option<Self> {
        Some(match x {
            0 => Self::NewGame,
            1 => Self::Continue,
            2 => Self::Exit,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ColorConfig {
    pub item: Phased<Color>,
    pub shadow: Phased<Color>,
}

impl Default for ColorConfig {
    fn default() -> Self {
        Self {
            item: Phased {
                selected: [24, 160, 120].into(),
                not_selected: [85, 40, 40].into(),
            },
            shadow: Phased {
                selected: [24, 24, 24].into(),
                not_selected: [16, 16, 16].into(),
            },
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TitleState {
    pub cursor: usize,
}

#[derive(Debug, PartialEq)]
pub struct TitleAssets {
    pub logo: SpriteData,
    pub choices: [SpriteData; 3],
    pub se_cursor: Asset<Wav>,
    pub se_select: Asset<Wav>,
    pub bgm: Asset<audio::src::WavStream>,
}

impl TitleAssets {
    /// TODO: use derive macro to load it
    pub fn new(assets: &mut AssetCacheAny) -> Self {
        Self {
            logo: title::Logo::load(assets).unwrap(),
            choices: title::Choices::load(assets).unwrap(),
            se_cursor: assets.load_sync(paths::sound::se::CURSOR).unwrap(),
            se_select: assets.load_sync(paths::sound::se::SELECT).unwrap(),
            bgm: assets.load_sync(paths::sound::bgm::FOREST_02).unwrap(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct TitleNodes {
    pub logo: Handle<Node>,
    /// [ [shadow, front ] ]
    pub choices: [[Handle<Node>; 2]; 3],
}

impl TitleNodes {
    pub fn new(cfg: &ColorConfig, nodes: &mut Pool<Node>, assets: &TitleAssets) -> Self {
        let logo = nodes.add(Draw::Sprite(assets.logo.clone()));

        let choices = [
            [
                nodes.add(Draw::Sprite(assets.choices[0].clone())),
                nodes.add(Draw::Sprite(assets.choices[0].clone())),
            ],
            [
                nodes.add(Draw::Sprite(assets.choices[1].clone())),
                nodes.add(Draw::Sprite(assets.choices[1].clone())),
            ],
            [
                nodes.add(Draw::Sprite(assets.choices[2].clone())),
                nodes.add(Draw::Sprite(assets.choices[2].clone())),
            ],
        ];

        // overwrite initial colors
        for i in 0..choices.len() {
            nodes[&choices[i][0]].params.color = cfg.shadow.not_selected;
            nodes[&choices[i][1]].params.color = cfg.item.not_selected;
        }

        Self { logo, choices }
    }
}

/// TODO: put animations in pool so that the animation remain when `Title` is poped
#[derive(Debug, PartialEq)]
pub struct TitleAnims {
    init_anims: Vec<Index<Anim>>,
}

impl TitleAnims {
    const DELTA_SHADOW: [f32; 2] = [10.0, 6.0];
    const DELTA_CHOICES: [[f32; 2]; 3] = [[0.0, 0.0], [100.0, 100.0], [320.0, 200.0]];

    fn logo_pos() -> Vec2f {
        [tweak!(440.0), tweak!(12.0)].into()
    }

    fn choice_pos(i: usize) -> Vec2f {
        Vec2f::new(tweak!(80.0), tweak!(350.0)).offset(Self::DELTA_CHOICES[i])
    }

    pub fn init(
        cfg: &ColorConfig,
        anims: &mut AnimStorage,
        nodes: &TitleNodes,
        cursor: usize,
    ) -> Self {
        let dt = ez::EasedDt::new(tweak!(1.2), ez::Ease::ExpOut);

        let mut gen = AnimGen::default();
        gen.node(&nodes.logo).dt(dt);

        let mut log = AnimInsertLog::bind(anims);
        log.insert(gen.pos([Self::logo_pos().offset([120.0, 6.0]), Self::logo_pos()]));
        log.insert(gen.color([Color::TRANSPARENT, Color::OPAQUE]));

        for i in 0..3 {
            // offset
            let from = Self::choice_pos(i).offset([tweak!(-120.0), tweak!(-30.0)]);
            let to = Self::choice_pos(i);

            gen.dt(dt);

            // shadow
            gen.node(&nodes.choices[i][0]);
            log.insert(gen.pos([
                from.offset(Self::DELTA_SHADOW),
                to.offset(Self::DELTA_SHADOW),
            ]));
            log.insert(gen.color([Color::TRANSPARENT, cfg.shadow.get(i == cursor)]));

            // text
            gen.node(&nodes.choices[i][1]);
            log.insert(gen.pos([from, to]));
            log.insert(gen.color([Color::TRANSPARENT, cfg.item.get(i == cursor)]));
        }

        Self {
            init_anims: log.into_vec(),
        }
    }

    pub fn select(
        &mut self,
        cfg: &ColorConfig,
        nodes: &TitleNodes,
        layer: &mut Layer,
        from: usize,
        to: usize,
    ) {
        // remove all the initial animations
        for ix in self.init_anims.drain(0..) {
            if let Some(anim) = layer.anims.get_mut(ix) {
                anim.set_accum_norm(1.0);
                anim.apply(&mut layer.nodes);
            }
            layer.anims.remove(ix);
        }

        let mut gen = AnimGen::default();
        gen.dt(EasedDt::new(6.0 / 60.0, Ease::Linear));

        // items
        layer.anims.insert(
            gen.node(&nodes.choices[from][1])
                .color([cfg.item.selected, cfg.item.not_selected]),
        );
        layer.anims.insert(
            gen.node(&nodes.choices[to][1])
                .color([cfg.item.not_selected, cfg.item.selected]),
        );

        // shadows
        layer.anims.insert(
            gen.node(&nodes.choices[from][0])
                .color([cfg.shadow.selected, cfg.shadow.not_selected]),
        );
        layer.anims.insert(
            gen.node(&nodes.choices[to][0])
                .color([cfg.shadow.not_selected, cfg.shadow.selected]),
        );
    }

    pub fn on_exit(&mut self, anims: &mut AnimStorage, nodes: &TitleNodes) {
        let mut gen = AnimGen::default();
        gen.dt(ez::EasedDt::new(24.0 / 60.0, ez::Ease::SinOut));

        gen.node(&nodes.logo);
        anims.insert(gen.pos([
            Self::logo_pos(),
            Self::logo_pos().offset([tweak!(-120.0), tweak!(-6.0)]),
        ]));
        anims.insert(gen.color([Color::OPAQUE, Color::TRANSPARENT]));

        for i in 0..3 {
            let pos = Self::choice_pos(i);

            // text
            gen.node(&nodes.choices[i][1]);
            anims.insert(gen.pos([pos, pos.offset([40.0, 20.0])]));
            anims.insert(gen.alpha([255, 0]));

            // shadow
            let pos = pos + Vec2f::from(Self::DELTA_SHADOW);
            gen.node(&nodes.choices[i][1]);
            anims.insert(gen.pos([pos, pos.offset(Self::DELTA_SHADOW).offset([40.0, 20.0])]));
            anims.insert(gen.alpha([255, 0]));
        }
    }
}
