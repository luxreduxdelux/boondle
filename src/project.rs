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
    exporter::{app_image::AppImage, debian::Debian, export::Export},
};
use std::fmt::Display;

//================================================================

use eframe::egui::{self, Color32};
use egui_modal::Modal;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

//================================================================

#[derive(Default, PartialEq, Eq)]
pub enum CompileStatus {
    #[default]
    InProgress,
    Success,
    Failure(String),
}

impl CompileStatus {
    pub fn color(&self) -> Color32 {
        match self {
            CompileStatus::InProgress => Color32::LIGHT_BLUE,
            CompileStatus::Success => Color32::LIGHT_GREEN,
            CompileStatus::Failure(_) => Color32::LIGHT_RED,
        }
    }
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

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Meta {
    pub path: PathBuf,
    pub name: String,
    pub icon: Option<String>,
    pub info: String,
    pub from: String,
    pub version: String,
    pub name_generic: String,
    pub comment: String,
    pub category: String,
    pub key_word: String,
    pub command_line: bool,
}

impl Meta {
    const FILE_DESKTOP: &str = r#"[Desktop Entry]
Name={name}
{icon}
Exec=/usr/bin/{name}
Type=Application
Categories=Utility;
"#;

    pub fn create_desktop_file(&self, icon_root: bool) -> String {
        let icon = if icon_root {
            format!("Icon=/{}-icon", &self.name)
        } else {
            format!("Icon=/usr/share/icons/{}-icon", &self.name)
        };

        let mut file = Self::FILE_DESKTOP.to_string();
        file = file.replace("{name}", &self.name);
        file = file.replace("{icon}", &icon);
        //file = file.replace("{name_generic}", &self.name_generic);
        //file = file.replace(
        //    "{command_line}",
        //    if self.command_line { "true" } else { "false" },
        //);
        //file = file.replace("{comment}", &self.comment);
        //file = file.replace("{category}", &self.category);
        //file = file.replace("{key_word}", &self.key_word);

        file
    }
}

#[derive(Default, Serialize, Deserialize)]
pub struct Project {
    pub meta: Meta,
    pub exporter: Vec<Box<dyn Export>>,
}

impl Project {
    pub fn new() -> anyhow::Result<Option<Self>> {
        if let Some(mut path) = rfd::FileDialog::new().save_file() {
            path.set_extension("json");

            println!("{path:?}");

            let result = Self {
                meta: Meta {
                    path: path.clone(),
                    ..Default::default()
                },
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
                    let _ = App::error(self.save(self.meta.path.clone()), "Save Error");
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
                ui.text_edit_singleline(&mut self.meta.name);

                App::pick_file(ui, "Icon", &mut self.meta.icon);

                ui.label("Info");
                ui.text_edit_singleline(&mut self.meta.info);

                ui.label("From");
                ui.text_edit_singleline(&mut self.meta.from);

                ui.label("Version");
                ui.text_edit_singleline(&mut self.meta.version);

                //App::pick_file(ui, "Linux Binary", &mut self.path_binary);

                //================================================================

                for exporter in &mut self.exporter {
                    exporter.poll_compile();
                }

                ui.heading("Compilation");
                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("+ Debian").clicked() {
                        self.exporter.push(Box::new(Debian::default()));
                    };

                    if ui.button("+ AppImage").clicked() {
                        self.exporter.push(Box::new(AppImage::default()));
                    };
                });

                ui.separator();

                for exporter in &mut self.exporter {
                    exporter.draw_setup(ui);
                }

                let can_export = self
                    .exporter
                    .iter_mut()
                    .any(|exporter| exporter.get_export());

                let modal = Modal::new(context, "my_modal");

                modal.show(|ui| {
                    modal.title(ui, "Export State");

                    modal.frame(ui, |ui| {
                        for exporter in &mut self.exporter {
                            exporter.draw_modal(ui);
                        }
                    });

                    let complete = self
                        .exporter
                        .iter_mut()
                        .all(|exporter| exporter.success_or_failure());

                    if complete {
                        modal.buttons(ui, |ui| {
                            if modal.button(ui, "Close").clicked() {};
                        });
                    }
                });

                if ui
                    .add_enabled(
                        !self.exporter.is_empty() && can_export,
                        egui::Button::new("Export"),
                    )
                    .clicked()
                {
                    if self.compile().is_ok() {
                        modal.open();
                    }
                }
            });
        });
    }

    fn compile(&mut self) -> anyhow::Result<()> {
        for exporter in &mut self.exporter {
            App::error(exporter.compile(self.meta.clone()), "Compile Error")?;
        }

        Ok(())
    }

    fn save(&self, path: PathBuf) -> anyhow::Result<()> {
        Ok(std::fs::write(path, serde_json::to_string_pretty(self)?)?)
    }

    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }
}
