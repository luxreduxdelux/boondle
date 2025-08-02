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
    exporter::export::*,
    project::{Meta, Project},
};

//================================================================

use eframe::egui::{self, CollapsingHeader, RichText};
use serde::{Deserialize, Serialize};

//================================================================

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind")]
enum Layout {
    String {
        name: String,
        data: String,
        #[serde(skip)]
        remove: bool,
    },
    Integer {
        name: String,
        data: i64,
        #[serde(skip)]
        remove: bool,
    },
    Decimal {
        name: String,
        data: f64,
        #[serde(skip)]
        remove: bool,
    },
    Boolean {
        name: String,
        data: bool,
        #[serde(skip)]
        remove: bool,
    },
}

#[derive(Default, Serialize, Deserialize)]
pub struct Script {
    name: String,
    script: Option<String>,
    layout: Vec<Layout>,
    enable: bool,
    #[serde(skip)]
    remove: bool,
    #[serde(skip)]
    status: ExportStatus,
    #[serde(skip)]
    handler: EventHandler,
}

#[typetag::serde]
impl Export for Script {
    fn draw_setup(&mut self, ui: &mut egui::Ui) {
        let header = CollapsingHeader::new(format_name("Custom Script", &self.name))
            .id_salt("custom_script");

        header.show(ui, |ui| {
            ui.checkbox(&mut self.enable, "Enable");

            ui.add_enabled_ui(self.enable, |ui| {
                Project::entry_label(ui, &mut self.name, "Name");

                App::pick_file(ui, "Script", &mut self.script);

                ui.separator();

                ui.horizontal(|ui| {
                    if ui.button("+ String").clicked() {
                        self.layout.push(Layout::String {
                            name: "String".to_string(),
                            data: "".to_string(),
                            remove: false,
                        });
                    }

                    if ui.button("+ Integer").clicked() {
                        self.layout.push(Layout::Integer {
                            name: "Integer".to_string(),
                            data: 0,
                            remove: false,
                        });
                    }

                    if ui.button("+ Decimal").clicked() {
                        self.layout.push(Layout::Decimal {
                            name: "Decimal".to_string(),
                            data: 0.0,
                            remove: false,
                        });
                    }

                    if ui.button("+ Boolean").clicked() {
                        self.layout.push(Layout::Boolean {
                            name: "Boolean".to_string(),
                            data: false,
                            remove: false,
                        });
                    }
                });

                self.layout.retain(|widget| match widget {
                    Layout::String { remove, .. } => !remove,
                    Layout::Integer { remove, .. } => !remove,
                    Layout::Decimal { remove, .. } => !remove,
                    Layout::Boolean { remove, .. } => !remove,
                });

                for widget in &mut self.layout {
                    match widget {
                        &mut Layout::String {
                            ref mut name,
                            ref mut data,
                            ref mut remove,
                        } => {
                            ui.label(name.as_str());
                            ui.text_edit_singleline(data).context_menu(|ui| {
                                ui.label("Name");
                                ui.text_edit_singleline(name);

                                if ui.button("Remove").clicked() {
                                    *remove = true;
                                }
                            });
                        }
                        &mut Layout::Integer {
                            ref mut name,
                            ref mut data,
                            ref mut remove,
                        } => {
                            ui.label(name.as_str());

                            ui.add(egui::DragValue::new(data).speed(0.1))
                                .context_menu(|ui| {
                                    ui.label("Name");
                                    ui.text_edit_singleline(name);

                                    if ui.button("Remove").clicked() {
                                        *remove = true;
                                    }
                                });
                        }
                        &mut Layout::Decimal {
                            ref mut name,
                            ref mut data,
                            ref mut remove,
                        } => {
                            ui.label(name.as_str());

                            ui.add(egui::DragValue::new(data).speed(0.1))
                                .context_menu(|ui| {
                                    ui.label("Name");
                                    ui.text_edit_singleline(name);

                                    if ui.button("Remove").clicked() {
                                        *remove = true;
                                    }
                                });
                        }
                        &mut Layout::Boolean {
                            ref mut name,
                            ref mut data,
                            ref mut remove,
                        } => {
                            ui.checkbox(data, name.as_str()).context_menu(|ui| {
                                ui.label("Name");
                                ui.text_edit_singleline(name);

                                if ui.button("Remove").clicked() {
                                    *remove = true;
                                }
                            });
                        }
                    }
                }
            });

            ui.separator();

            if ui.button("Remove").clicked() {
                self.remove = true;
            }
        });
    }

    fn draw_modal(&mut self, ui: &mut egui::Ui) {
        if self.enable {
            ui.horizontal(|ui| {
                ui.label(format_name("Custom Script", &self.name));
                ui.label(RichText::new(format!("{}", self.status)).color(self.status.color()));

                if self.status == ExportStatus::InProgress {
                    ui.spinner();
                }
            });
        }
    }

    fn get_enable(&self) -> bool {
        self.enable
    }

    fn get_remove(&self) -> bool {
        self.remove
    }

    fn get_status(&mut self) -> &mut ExportStatus {
        &mut self.status
    }

    fn get_handler(&mut self) -> &mut EventHandler {
        &mut self.handler
    }

    fn run(&mut self, meta: Meta) -> anyhow::Result<()> {
        if !self.enable {
            return Ok(());
        }

        self.status = ExportStatus::InProgress;

        if meta.name.is_empty() {
            return Err(anyhow::Error::msg(
                "Custom Script: Project name cannot be empty.",
            ));
        }

        if meta.version.is_empty() {
            return Err(anyhow::Error::msg(
                "Custom Script: Project version cannot be empty.",
            ));
        }

        if let Some(script) = &self.script {
            let mut command = std::process::Command::new(script);

            // TO-DO move into own function?
            command.env("BOONDLE_NAME", meta.name);
            command.env("BOONDLE_ICON", meta.icon.unwrap_or_default());
            command.env("BOONDLE_INFO", meta.info);
            command.env("BOONDLE_FROM", meta.from);
            command.env("BOONDLE_VERSION", meta.version);
            command.env("BOONDLE_NAME_GENERIC", meta.name_generic);
            command.env("BOONDLE_NAME_COMMENT", meta.comment);
            command.env("BOONDLE_NAME_CATEGORY", meta.category);
            command.env("BOONDLE_NAME_KEY_WORD", meta.key_word);

            for widget in &self.layout {
                match widget {
                    Layout::String { name, data, .. } => command.env(name.to_uppercase(), data),
                    Layout::Integer { name, data, .. } => {
                        command.env(name.to_uppercase(), data.to_string())
                    }
                    Layout::Decimal { name, data, .. } => {
                        command.env(name.to_uppercase(), data.to_string())
                    }
                    Layout::Boolean { name, data, .. } => {
                        command.env(name.to_uppercase(), if *data { "1" } else { "0" })
                    }
                };
            }

            self.execute(command);
        } else {
            self.status = ExportStatus::Success;
        }

        Ok(())
    }
}
