pub fn build_conf() -> macroquad::window::Conf {
    macroquad::window::Conf {
        window_title: "Phi TK".to_string(),
        window_width: 1280,
        window_height: 720,
        headless: std::env::args().skip(1).next().as_deref() != Some("preview"),
        ..Default::default()
    }
}

#[macroquad::main(build_conf)]
async fn main() -> Result<(), anyhow::Error> {
    app_lib::run().await?;
    Ok(())
}
