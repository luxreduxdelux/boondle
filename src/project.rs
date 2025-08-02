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
    exporter::{app_image::AppImage, debian::Debian, export::Export, script::Script},
};

//================================================================

use eframe::egui::{self, Response};
use egui_modal::Modal;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

//================================================================

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
    pub compile: Vec<Box<dyn Export>>,
    pub package: Vec<Box<dyn Export>>,
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
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        todo!()
                    };

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
        });

        egui::CentralPanel::default().show(context, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                self.draw_project(ui);
                self.draw_compile(ui);
                self.draw_package(ui);

                //================================================================

                let modal_compile = Modal::new(context, "modal_compile");

                modal_compile.show(|ui| {
                    modal_compile.title(ui, "Compile");

                    modal_compile.frame(ui, |ui| {
                        for compile in &mut self.compile {
                            compile.draw_modal(ui);
                        }
                    });

                    let complete_compile = self
                        .compile
                        .iter_mut()
                        .all(|compile| compile.success_or_failure());

                    if complete_compile {
                        modal_compile.buttons(ui, |ui| {
                            if modal_compile.button(ui, "Close").clicked() {};
                        });
                    }
                });

                //================================================================

                let modal_package = Modal::new(context, "modal_package");

                modal_package.show(|ui| {
                    modal_compile.title(ui, "Package");

                    modal_package.frame(ui, |ui| {
                        for package in &mut self.package {
                            package.draw_modal(ui);
                        }
                    });

                    let complete_package = self
                        .package
                        .iter_mut()
                        .all(|package| package.success_or_failure());

                    if complete_package {
                        modal_package.buttons(ui, |ui| {
                            if modal_package.button(ui, "Close").clicked() {};
                        });
                    }
                });

                //================================================================

                let can_compile = !self.compile.is_empty()
                    && self.compile.iter_mut().any(|compile| compile.get_enable());
                let can_package = !self.package.is_empty()
                    && self.package.iter_mut().any(|package| package.get_enable());

                if Self::button_enable(ui, can_compile, "Compile").clicked()
                    && self.compile().is_ok()
                {
                    modal_compile.open();
                }

                if Self::button_enable(ui, can_package, "Package").clicked()
                    && self.package().is_ok()
                {
                    modal_package.open();
                }
            });
        });
    }

    fn entry_label(ui: &mut egui::Ui, text: &mut String, label: &str) {
        ui.label(label);
        ui.text_edit_singleline(text);
    }

    fn button_enable(ui: &mut egui::Ui, enable: bool, label: &str) -> Response {
        ui.add_enabled(enable, egui::Button::new(label))
    }

    pub fn compile(&mut self) -> anyhow::Result<()> {
        for compile in &mut self.compile {
            App::error(compile.run(self.meta.clone()), "Compile Error")?;
        }

        Ok(())
    }

    pub fn package(&mut self) -> anyhow::Result<()> {
        for package in &mut self.package {
            App::error(package.run(self.meta.clone()), "Compile Error")?;
        }

        Ok(())
    }

    pub fn load(path: PathBuf) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
    }

    fn save(&self, path: PathBuf) -> anyhow::Result<()> {
        Ok(std::fs::write(path, serde_json::to_string_pretty(self)?)?)
    }

    #[rustfmt::skip]
    fn draw_project(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("Project", |ui| {
            Self::entry_label(ui, &mut self.meta.name,         "Name");
            Self::entry_label(ui, &mut self.meta.info,         "Info");
            Self::entry_label(ui, &mut self.meta.from,         "From");
            Self::entry_label(ui, &mut self.meta.version,      "Version");
            Self::entry_label(ui, &mut self.meta.name_generic, "Generic Name");
            Self::entry_label(ui, &mut self.meta.comment,      "Comment");
            Self::entry_label(ui, &mut self.meta.category,     "Category");
            Self::entry_label(ui, &mut self.meta.key_word,     "Key-Word");

            App::pick_file(ui, "Icon", &mut self.meta.icon);

            ui.checkbox(&mut self.meta.command_line, "Command-Line Application");
        });
    }

    fn draw_compile(&mut self, ui: &mut egui::Ui) {
        for compile in &mut self.compile {
            compile.poll_completion();
        }

        ui.collapsing("Compile", |ui| {
            if ui.button("+ Custom Script").clicked() {
                self.compile.push(Box::new(Script::default()));
            };

            ui.separator();

            for (i, compile) in self.compile.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    compile.draw_setup(ui);
                });
            }
        });

        self.compile.retain(|compile| !compile.get_remove());
    }

    fn draw_package(&mut self, ui: &mut egui::Ui) {
        for package in &mut self.package {
            package.poll_completion();
        }

        ui.collapsing("Package", |ui| {
            ui.horizontal(|ui| {
                if ui.button("+ Debian").clicked() {
                    self.package.push(Box::new(Debian::default()));
                };

                if ui.button("+ AppImage").clicked() {
                    self.package.push(Box::new(AppImage::default()));
                };

                if ui.button("+ Custom Script").clicked() {
                    self.package.push(Box::new(Script::default()));
                };
            });

            ui.separator();

            for (i, package) in self.package.iter_mut().enumerate() {
                ui.push_id(i, |ui| {
                    package.draw_setup(ui);
                });
            }
        });

        self.package.retain(|package| !package.get_remove());
    }
}
