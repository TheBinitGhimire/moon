mod frame;
mod loader;
mod page;
mod renderer;

use gfx::Bitmap;
use renderer::{Renderer, RendererInitializeParams};
use url::Url;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

pub async fn render_once(html: String, base_url: Url, size: (u32, u32)) -> Bitmap {
    let mut renderer = Renderer::new().await;

    renderer.initialize(RendererInitializeParams { viewport: size });

    renderer.load_html(html, base_url);

    renderer.paint();

    renderer.output().await
}
