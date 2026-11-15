use crate::{models::nekosbest::NekosBestData, nekosbest::NekosBest};
use eframe::egui;
use kitties::Client;
use std::{
  path::PathBuf,
  sync::mpsc::{Receiver, Sender, channel},
};

enum Message {
  Model(NekosBestData),
  Error(String),
}

#[derive(PartialEq)]
enum ImageType {
  Neko,
  Waifu,
}

pub struct Kitties {
  client: NekosBest,
  model: Option<NekosBestData>,
  error: Option<String>,
  image_type: ImageType,

  sender: Sender<Message>,
  receiver: Receiver<Message>,
}

impl Default for Kitties {
  fn default() -> Self {
    let (sender, receiver) = channel();
    Self {
      client: NekosBest::new("https://nekos.best/api/v2/"),
      model: None,
      error: None,
      image_type: ImageType::Neko,
      sender,
      receiver,
    }
  }
}

impl Kitties {
  fn fetch(&mut self, ctx: &egui::Context) {
    let sender = self.sender.clone();
    let client = self.client.clone();
    let ctx = ctx.clone();

    let endpoint = match self.image_type {
      ImageType::Neko => "neko".to_string(),
      ImageType::Waifu => "waifu".to_string(),
    };

    tokio::spawn(async move {
      let msg = match client.get(&endpoint).await {
        Ok(api) => match api.results.first() {
          Some(data) => Message::Model(data.clone()),
          None => Message::Error("The API returned no results.".into()),
        },
        Err(e) => Message::Error(format!("Could not get kitty: {e}")),
      };

      let _ = sender.send(msg);
      ctx.request_repaint();
    });
  }

  fn save(&mut self, ctx: egui::Context, path: PathBuf) {
    let model = self.model.clone();
    let sender = self.sender.clone();
    let client = self.client.clone();

    tokio::spawn(async move {
      let download: Result<(), String> = async {
        let data = model.ok_or("No image currently loaded.".to_string())?;

        let response = client
          .http()
          .get(&data.url)
          .send()
          .await
          .map_err(|err| format!("Could not request image: {err}"))?;

        let valid = response
          .error_for_status()
          .map_err(|err| format!("Could not load image: {err}"))?;

        let bytes = valid
          .bytes()
          .await
          .map_err(|err| format!("Could not get image bytes: {err}"))?;

        std::fs::write(&path, bytes).map_err(|err| format!("Could not save image: {err}"))?;

        Ok(())
      }
      .await;

      if let Err(err) = download {
        let _ = sender.send(Message::Error(err));
      }

      ctx.request_repaint();
    });
  }
}

impl eframe::App for Kitties {
  fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
    while let Ok(msg) = self.receiver.try_recv() {
      match msg {
        Message::Model(model) => self.model = Some(model),
        Message::Error(e) => self.error = Some(e),
      }
    }

    let shortcut_new = egui::KeyboardShortcut::new(egui::Modifiers::COMMAND, egui::Key::N);
    let shortcut_save = egui::KeyboardShortcut::new(
      egui::Modifiers::COMMAND | egui::Modifiers::SHIFT,
      egui::Key::S,
    );

    let mut trigger_new = false;
    let mut trigger_save = false;

    ui.input_mut(|i| {
      if i.consume_shortcut(&shortcut_new) {
        trigger_new = true;
      }

      if i.consume_shortcut(&shortcut_save) {
        trigger_save = true;
      }
    });

    egui::Panel::top("top_panel").show(ui, |ui| {
      egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
          if ui.button("New Image (Ctrl + N)").clicked() {
            trigger_new = true;
          }

          if ui.button("Save As... (Ctrl + Shift + S)").clicked() {
            trigger_save = true;
          }
        });

        //ui.menu_button("History", |ui| {
        //if ui.button("Add to collection").clicked() {}
        //if ui.button("View collection").clicked() {}
        //});

        ui.menu_button("Image", |ui| {
          if ui
            .radio_value(&mut self.image_type, ImageType::Waifu, "Waifu")
            .clicked()
          {
            self.fetch(ui.ctx());
          }

          if ui
            .radio_value(&mut self.image_type, ImageType::Neko, "Neko")
            .clicked()
          {
            self.fetch(ui.ctx());
          }
        });
      });
    });

    if trigger_new {
      self.fetch(ui.ctx());
    }

    if trigger_save {
      if let Some(path) = rfd::FileDialog::new()
        .add_filter("PNG Images", &["png"])
        .set_file_name(format!("{}.png", uuid::Uuid::new_v4().to_string()))
        .save_file()
      {
        self.save(ui.ctx().clone(), path);
      }
    }

    if let Some(model) = &self.model {
      egui::Area::new(egui::Id::unique("artist_area"))
        .anchor(egui::Align2::RIGHT_BOTTOM, egui::vec2(-10.0, -10.0))
        .show(ui.ctx(), |ui| {
          egui::Frame::new()
            .fill(egui::Color32::from_black_alpha(180))
            .inner_margin(10.0)
            .show(ui, |ui| {
              ui.add(
                egui::Label::new(egui::RichText::new(format!(
                  "Artist: {}",
                  &model.artist_name
                )))
                .wrap_mode(egui::TextWrapMode::Extend),
              );
              ui.horizontal(|ui| {
                ui.label("Source:");
                ui.hyperlink_to("Website", &model.source_url)
              });
            })
        });
    }
    egui::CentralPanel::default().show(ui, |ui| {
      ui.centered_and_justified(|ui| {
        ui.vertical_centered(|ui| match &self.model {
          Some(model) => {
            ui.add(
              egui::Image::new(&model.url)
                .max_width(800.0)
                .max_height(1000.0),
            );
          }
          None => {
            self.fetch(ui.ctx());
          }
        });
      });
    });

    if let Some(err) = &self.error {
      let modal = egui::Modal::new(egui::Id::unique("error_modal")).show(ui.ctx(), |ui| {
        ui.heading("Something went wrong");
        ui.add_space(4.0);
        ui.label(err);
        ui.add_space(8.0);
        if ui.button("OK").clicked() {
          ui.close();
        }
      });

      if modal.should_close() {
        self.error = None;
      }
    }
  }
}
