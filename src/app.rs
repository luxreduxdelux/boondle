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

use crate::{project::*, setting::*};

//================================================================

use eframe::egui;

//================================================================

#[derive(Default)]
pub struct App {
    pub project: Option<Project>,
    pub setting: Setting,
}

impl App {
    pub fn new(context: &eframe::CreationContext<'_>) -> Self {
        context.egui_ctx.set_zoom_factor(1.25);

        Self::default()
    }

    pub fn pick_file(ui: &mut egui::Ui, name: &str, path: &mut Option<String>) {
        ui.horizontal(|ui| {
            if ui.button(name).clicked()
                && let Some(file) = rfd::FileDialog::new().pick_file()
            {
                *path = Some(file.display().to_string());
            }

            if let Some(path) = &path {
                ui.label(format!("Path: {path}"));
            } else {
                ui.label("Path: <none>");
            }
        });
    }

    pub fn error<T>(result: anyhow::Result<T>, title: &str) -> anyhow::Result<T> {
        if let Err(ref error) = result {
            rfd::MessageDialog::new()
                .set_title(title)
                .set_level(rfd::MessageLevel::Error)
                .set_description(error.to_string())
                .show();
        }

        result
    }
}

impl eframe::App for App {
    fn update(&mut self, context: &egui::Context, _: &mut eframe::Frame) {
        if let Some(project) = &mut self.project {
            project.draw(context);
        } else {
            let width = context.screen_rect();

            egui::TopBottomPanel::top("panel_a").show(context, |ui| {
                ui.heading("Boondle");
            });

            egui::SidePanel::left("panel_b")
                .exact_width(width.width() / 2.0)
                .show(context, |ui| {
                    ui.label("New/Load");

                    if ui.button("New Project").clicked()
                        && let Ok(Some(project)) = Self::error(Project::new(), "New Error")
                    {
                        self.setting.history_add(project.meta.path.clone());
                        self.project = Some(project);
                    }

                    if ui.button("Load Project").clicked()
                        && let Some(path) = rfd::FileDialog::new().pick_file()
                        && let Ok(project) = Self::error(Project::load(path), "Load Error")
                    {
                        self.setting.history_add(project.meta.path.clone());
                        self.project = Some(project);
                    };
                });

            egui::SidePanel::right("panel_c")
                .exact_width(width.width() / 2.0)
                .show(context, |ui| {
                    ui.label("Recent");

                    for path in &self.setting.history {
                        if ui.button(path.to_str().unwrap()).clicked()
                            && let Ok(project) =
                                Self::error(Project::load(path.into()), "Load Error")
                        {
                            self.project = Some(project);
                        }
                    }
                });
        }
    }
}
