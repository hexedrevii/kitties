use eframe::egui;

mod kitties;
mod models;
mod nekosbest;

#[tokio::main]
async fn main() -> eframe::Result {
  let options = eframe::NativeOptions {
    viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
    ..Default::default()
  };
  eframe::run_native(
    "Kitties",
    options,
    Box::new(|cc| {
      egui_extras::install_image_loaders(&cc.egui_ctx);
      Ok(Box::<kitties::Kitties>::default())
    }),
  )
}
