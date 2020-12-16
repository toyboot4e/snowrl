use snow_rl::SnowRl;

fn main() -> rokol::Result {
    env_logger::init();

    let rokol = rokol::Rokol {
        w: 1280,
        h: 720,
        title: "SnowRL".to_string(),
        ..Default::default()
    };

    let mut app = SnowRl::new();

    rokol.run(&mut app)
}
