/*!
Scenes
*/

use snow2d::audio::src::Wav;

use rlbox::ui::{
    anims::*,
    node::{Draw, Node},
    AnimPool, Ui,
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

#[derive(Debug, Clone)]
pub struct Phased<T> {
    pub selected: T,
    pub not_selected: T,
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

#[derive(Debug, Clone)]
pub struct ItemConfig {
    pub item: Phased<Color>,
    pub shadow: Phased<Color>,
}

impl Default for ItemConfig {
    fn default() -> Self {
        Self {
            item: Phased {
                selected: Color {
                    r: 24,
                    g: 160,
                    b: 120,
                    a: 255,
                },

                not_selected: Color {
                    r: 85,
                    g: 40,
                    b: 40,
                    a: 255,
                },
            },

            shadow: Phased {
                selected: Color {
                    r: 24,
                    g: 24,
                    b: 24,
                    a: 255,
                },

                not_selected: Color {
                    r: 16,
                    g: 16,
                    b: 16,
                    a: 255,
                },
            },
        }
    }
}

#[derive(Debug)]
pub struct TitleState {
    pub cursor: usize,
}

#[derive(Debug)]
pub struct TitleAssets {
    pub cfg: ItemConfig,
    pub logo: SpriteData,
    pub choices: [SpriteData; 3],
    pub se_cursor: Asset<Wav>,
    pub se_select: Asset<Wav>,
    pub bgm: Asset<audio::src::WavStream>,
}

impl TitleAssets {
    pub fn new(cfg: ItemConfig, assets: &mut AssetCacheAny) -> Self {
        Self {
            cfg,
            logo: title::Logo::load(assets).unwrap(),
            choices: title::Choices::load(assets).unwrap(),
            se_cursor: assets.load_sync(paths::sound::se::CURSOR).unwrap(),
            se_select: assets.load_sync(paths::sound::se::SELECT).unwrap(),
            bgm: assets.load_sync(paths::sound::bgm::FOREST_02).unwrap(),
        }
    }
}

#[derive(Debug)]
pub struct TitleNodes {
    pub logo: Handle<Node>,
    /// [ [shadow, front ] ]
    pub choices: [[Handle<Node>; 2]; 3],
}

impl TitleNodes {
    pub fn new(nodes: &mut Pool<Node>, assets: &TitleAssets) -> Self {
        let logo = nodes.add(Draw::Sprite(assets.logo.clone()).into());

        let choices = [
            [
                nodes.add(Draw::Sprite(assets.choices[0].clone()).into()),
                nodes.add(Draw::Sprite(assets.choices[0].clone()).into()),
            ],
            [
                nodes.add(Draw::Sprite(assets.choices[1].clone()).into()),
                nodes.add(Draw::Sprite(assets.choices[1].clone()).into()),
            ],
            [
                nodes.add(Draw::Sprite(assets.choices[2].clone()).into()),
                nodes.add(Draw::Sprite(assets.choices[2].clone()).into()),
            ],
        ];

        // overwrite initial colors
        for i in 0..choices.len() {
            nodes[&choices[i][0]].params.color = assets.cfg.shadow.not_selected;
            nodes[&choices[i][1]].params.color = assets.cfg.item.not_selected;
        }

        Self { logo, choices }
    }
}

/// TODO: put animations in pool so that the animation remain when `Title` is poped
#[derive(Debug)]
pub struct TitleAnims {
    logo: Index<Anim>,
    choices: Vec<Index<Anim>>,
}

impl TitleAnims {
    pub fn init(cfg: &ItemConfig, anims: &mut AnimPool, nodes: &TitleNodes, cursor: usize) -> Self {
        let dt = ez::EasedDt::new(tweak!(1.2), ez::Ease::ExpOut);

        let logo = anims.insert(PosTween {
            tween: ez::Tweened {
                a: [tweak!(560.0), tweak!(18.0)].into(),
                b: [tweak!(440.0), tweak!(12.0)].into(),
                dt,
            },
            node: nodes.logo.clone(),
        });

        anims.insert(ColorTween {
            tween: ez::Tweened {
                a: Color::TRANSPARENT,
                b: Color::OPAQUE,
                dt,
            },
            node: nodes.logo.clone(),
        });

        let a = Vec2f::new(tweak!(200.0), tweak!(380.0));
        let b = Vec2f::new(tweak!(80.0), tweak!(350.0));
        let dt = ez::EasedDt::new(tweak!(1.2), ez::Ease::ExpOut);

        let mut choices = vec![];

        for i in 0..3 {
            let [[a, b], [c, d]] = Self::choice_anim(
                cfg,
                nodes,
                (a, Color::TRANSPARENT),
                (b, Color::WHITE),
                dt.clone(),
                i,
                cursor,
            );

            choices.push(anims.insert(a));
            choices.push(anims.insert(b));
            choices.push(anims.insert(c));
            choices.push(anims.insert(d));
        }

        Self { logo, choices }
    }

    pub fn select(
        &mut self,
        cfg: &ItemConfig,
        nodes: &TitleNodes,
        anims: &mut AnimPool,
        from: usize,
        to: usize,
    ) {
        let dt = EasedDt::new(6.0 / 60.0, Ease::Linear);

        // items
        let fade_out = ColorTween {
            tween: ez::Tweened {
                a: cfg.item.selected,
                b: cfg.item.not_selected,
                dt,
            },
            node: nodes.choices[from][1].clone(),
        };

        let fade_in = ColorTween {
            tween: ez::Tweened {
                a: cfg.item.not_selected,
                b: cfg.item.selected,
                dt,
            },
            node: nodes.choices[to][1].clone(),
        };

        anims.insert(fade_out);
        anims.insert(fade_in);

        // shadows
        let fade_out = ColorTween {
            tween: ez::Tweened {
                a: cfg.shadow.selected,
                b: cfg.shadow.not_selected,
                dt,
            },
            node: nodes.choices[from][0].clone(),
        };

        let fade_in = ColorTween {
            tween: ez::Tweened {
                a: cfg.shadow.not_selected,
                b: cfg.shadow.selected,
                dt,
            },
            node: nodes.choices[to][0].clone(),
        };

        anims.insert(fade_out);
        anims.insert(fade_in);
    }

    pub fn on_exit(&mut self, ui: &mut Ui, nodes: &TitleNodes) {
        let dt = ez::EasedDt::new(24.0 / 60.0, ez::Ease::SinOut);

        ui.anims.insert(PosTween {
            tween: ez::Tweened {
                a: [tweak!(440.0), tweak!(18.0)].into(),
                b: [tweak!(320.0), tweak!(6.0)].into(),
                dt,
            },
            node: nodes.logo.clone(),
        });

        ui.anims.insert(ColorTween {
            tween: ez::Tweened {
                // TODO: x or y only tween
                a: Color::OPAQUE,
                b: Color::TRANSPARENT,
                dt,
            },
            node: nodes.logo.clone(),
        });
    }
}

impl TitleAnims {
    fn choice_anim(
        cfg: &ItemConfig,
        nodes: &TitleNodes,
        from: (Vec2f, Color),
        to: (Vec2f, Color),
        dt: ez::EasedDt,
        i: usize,
        cursor: usize,
    ) -> [[Anim; 2]; 2] {
        let delta_shadow = Vec2f::new(10.0, 6.0);
        let delta_choices: [Vec2f; 3] = [
            [0.0, 0.0].into(),
            [100.0, 100.0].into(),
            [320.0, 200.0].into(),
        ];

        let shadow_pos = PosTween {
            tween: ez::Tweened {
                a: delta_shadow + from.0 + delta_choices[i],
                b: delta_shadow + to.0 + delta_choices[i],
                dt,
            },
            node: nodes.choices[i][0].clone(),
        };

        let shadow_color = ColorTween {
            tween: ez::Tweened {
                a: Color::TRANSPARENT,
                b: if i == cursor {
                    cfg.shadow.selected
                } else {
                    cfg.shadow.not_selected
                },
                dt,
            },
            node: nodes.choices[i][0].clone(),
        };

        let item_pos = PosTween {
            tween: ez::Tweened {
                a: from.0 + delta_choices[i],
                b: to.0 + delta_choices[i],
                dt,
            },
            node: nodes.choices[i][1].clone(),
        };

        let item_color = ColorTween {
            tween: ez::Tweened {
                a: Color::TRANSPARENT,
                b: if i == cursor {
                    cfg.item.selected
                } else {
                    cfg.item.not_selected
                },
                dt,
            },
            node: nodes.choices[i][1].clone(),
        };

        [
            [shadow_pos.into(), shadow_color.into()],
            [item_pos.into(), item_color.into()],
        ]
    }
}
