fn main() -> rokol::Result {
    env_logger::init();

    let rokol = rokol::Rokol {
        w: 1280,
        h: 720,
        title: "SnowRL".to_string(),
        use_high_dpi: false,
        ..Default::default()
    };

    snowrl::run(rokol)
}
