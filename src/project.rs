/*
* Copyright (c) 2025 luxreduxdelux
*
* Redistribution and use in source and binary forms, with or without
* modification, are permitted provided that the following conditions are met:
*
* 1. Redistributions of source code must retain the above copyright notice,
* this list of conditions and the following disclaimer.
*
* 2. Redistributions in binary form must reproduce the above copyright notice,
* this list of conditions and the following disclaimer in the documentation
* and/or other materials provided with the distribution.
*
* Subject to the terms and conditions of this license, each copyright holder
* and contributor hereby grants to those receiving rights under this license
* a perpetual, worldwide, non-exclusive, no-charge, royalty-free, irrevocable
* (except for failure to satisfy the conditions of this license) patent license
* to make, have made, use, offer to sell, sell, import, and otherwise transfer
* this software, where such license applies only to those patent claims, already
* acquired or hereafter acquired, licensable by such copyright holder or
* contributor that are necessarily infringed by:
*
* (a) their Contribution(s) (the licensed copyrights of copyright holders and
* non-copyrightable additions of contributors, in source or binary form) alone;
* or
*
* (b) combination of their Contribution(s) with the work of authorship to which
* such Contribution(s) was added by such copyright holder or contributor, if,
* at the time the Contribution is added, such addition causes such combination
* to be necessarily infringed. The patent license shall not apply to any other
* combinations which include the Contribution.
*
* Except as expressly stated above, no rights or licenses from any copyright
* holder or contributor is granted under this license, whether expressly, by
* implication, estoppel or otherwise.
*
* DISCLAIMER
*
* THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
* AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
* IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
* DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDERS OR CONTRIBUTORS BE LIABLE
* FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
* DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
* SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
* CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
* OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
* OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
*/

use crate::{
    app::App,
    exporter::{app_image::AppImage, debian::Debian},
};
use std::{fmt::Display, sync::mpsc::channel};

//================================================================

use eframe::egui;
use egui_modal::Modal;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

//================================================================

#[derive(Default)]
pub enum CompileStatus {
    #[default]
    InProgress,
    Success,
    Failure(String),
}

impl Display for CompileStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileStatus::InProgress => f.write_str("In Progress"),
            CompileStatus::Success => f.write_str("Success"),
            CompileStatus::Failure(error) => f.write_str(&format!("Failure: {error}")),
        }
    }
}

pub enum Event {
    Debian(anyhow::Result<()>),
    AppImage(anyhow::Result<()>),
}

#[derive(Serialize, Deserialize)]
pub struct Project {
    pub path: PathBuf,
    pub name: String,
    pub icon: Option<String>,
    pub info: String,
    pub from: String,
    pub version: String,
    pub path_binary: Option<String>,
    pub debian: Debian,
    pub app_image: AppImage,
    #[serde(skip)]
    pub event_tx: Option<Sender<Event>>,
    #[serde(skip)]
    pub event_rx: Option<Receiver<Event>>,
}

impl Default for Project {
    fn default() -> Self {
        let (event_tx, event_rx) = channel();

        Self {
            path: PathBuf::default(),
            name: String::default(),
            icon: None,
            info: String::default(),
            from: String::default(),
            version: String::default(),
            path_binary: None,
            debian: Debian::default(),
            app_image: AppImage::default(),
            event_tx: Some(event_tx),
            event_rx: Some(event_rx),
        }
    }
}

impl Project {
    pub fn new() -> anyhow::Result<Option<Self>> {
        if let Some(path) = rfd::FileDialog::new().save_file() {
            let result = Self {
                path: path.clone(),
                ..Default::default()
            };

            result.save(path)?;

            return Ok(Some(result));
        }

        Ok(None)
    }

    pub fn draw(&mut self, context: &egui::Context) {
        egui::TopBottomPanel::top("layout").show(context, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    let _ = App::error(self.save(self.path.clone()), "Load Error");
                };

                if ui.button("Load").clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_file()
                    && let Ok(data) = App::error(Self::load(path), "Load Error")
                {
                    *self = data;
                };
            });
        });

        egui::CentralPanel::default().show(context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                ui.heading("Project Information");
                ui.separator();

                ui.label("Name");
                ui.text_edit_singleline(&mut self.name);

                App::pick_file(ui, "Icon", &mut self.icon);

                ui.label("Info");
                ui.text_edit_singleline(&mut self.info);

                ui.label("From");
                ui.text_edit_singleline(&mut self.from);

                ui.label("Version");
                ui.text_edit_singleline(&mut self.version);

                App::pick_file(ui, "Linux Binary", &mut self.path_binary);

                //================================================================

                let event = self.event_rx.as_ref().unwrap();

                if let Ok(event) = event.try_recv() {
                    match event {
                        Event::Debian(result) => {
                            if let Err(error) = result {
                                self.debian.status = CompileStatus::Failure(error.to_string());
                            } else {
                                self.debian.status = CompileStatus::Success;
                            }
                        }
                        Event::AppImage(result) => {
                            if let Err(error) = result {
                                self.app_image.status = CompileStatus::Failure(error.to_string());
                            } else {
                                self.app_image.status = CompileStatus::Success;
                            }
                        }
                    }
                }

                ui.heading("Compilation");
                ui.separator();

                self.debian.draw(ui);
                self.app_image.draw(ui);

                let modal = Modal::new(context, "my_modal");

                modal.show(|ui| {
                    modal.title(ui, "Export State");

                    modal.frame(ui, |ui| {
                        ui.label(format!(".deb package: {}", self.debian.status));
                        ui.label(format!(".app package: {}", self.app_image.status));
                    });

                    modal.buttons(ui, |ui| {
                        if modal.button(ui, "Close").clicked() {};
                    });
                });

                if ui.button("Export").clicked() {
                    modal.open();
                    let _ = self.export();
                }
            });
        });
    }

    fn export(&mut self) -> anyhow::Result<()> {
        App::error(Debian::export(self), "Debian Error")?;
        App::error(AppImage::export(self), "AppImage Error")?;

        Ok(())
    }

    fn save(&self, path: PathBuf) -> anyhow::Result<()> {
        let data: Vec<u8> = postcard::to_allocvec(self)?;
        std::fs::write(path, data)?;

        Ok(())
    }

    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        let data = std::fs::read(path)?;
        let data = postcard::from_bytes(&data)?;

        let (event_tx, event_rx) = channel();

        Ok(Self {
            event_tx: Some(event_tx),
            event_rx: Some(event_rx),
            ..data
        })
    }
}
