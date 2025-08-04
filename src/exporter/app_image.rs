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

#[derive(Default, Serialize, Deserialize)]
pub struct AppImage {
    name: String,
    file: String,
    binary: String,
    script: String,
    enable: bool,
    #[serde(skip)]
    remove: bool,
    #[serde(skip)]
    status: ExportStatus,
    #[serde(skip)]
    handler: EventHandler,
}

#[typetag::serde]
impl Export for AppImage {
    fn draw_setup(&mut self, ui: &mut egui::Ui) {
        let header = CollapsingHeader::new(format_name("AppImage (.AppImage)", &self.name))
            .id_salt("app_image");

        header.show(ui, |ui| {
            ui.checkbox(&mut self.enable, "Enable");

            ui.add_enabled_ui(self.enable, |ui| {
                Project::entry_label(ui, &mut self.name, "Name");
                Project::entry_label(ui, &mut self.file, "File");

                Project::pick_file(ui, "Binary", &mut self.binary);
                Project::pick_file(ui, "After-Installation Script", &mut self.script);
                //self.desktop.draw(ui);
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
                ui.label(format_name("AppImage (.AppImage)", &self.name));
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
                "AppImage: Project name cannot be empty.",
            ));
        }

        let work = format!("boondle_app_image/{}.AppDir", meta.name);
        let usr = format!("{work}/usr");

        // create boondle_app_image folder.
        std::fs::create_dir_all(format!("boondle_app_image"))?;

        // create work folder.
        std::fs::create_dir_all(&work)?;

        //================================================================

        // copy after-install script.
        if !self.script.is_empty() {
            std::fs::copy(&self.script, format!("{work}/AppRun"))?;
        } else {
            // write AppRun file.
            std::fs::write(format!("{work}/AppRun"), Self::file_app_run(&meta))?;
        }

        std::process::Command::new("chmod")
            .arg("a+x")
            .arg(format!("{work}/AppRun"))
            .output()?;

        // write .desktop file.
        std::fs::write(
            format!("{work}/{}.desktop", meta.name),
            meta.create_desktop_file(true),
        )?;

        //================================================================

        // create binary folder.
        std::fs::create_dir_all(format!("{usr}/bin"))?;

        // copy binary, if present.
        if !self.binary.is_empty() {
            std::fs::copy(&self.binary, format!("{usr}/bin/{}", meta.name))?;
        }

        // copy icon file, if present.
        if !meta.icon.is_empty() {
            // appimagetool won't work if we don't have an extension at the end...
            std::fs::copy(&meta.icon, format!("{work}/{}-icon.png", meta.name))?;
        }

        //================================================================

        let path = if self.file.is_empty() {
            format!(
                "{}_{}{}.AppImage",
                meta.name,
                meta.version,
                format_name_present(&self.name)
            )
        } else {
            format!("{}.AppImage", format_file(&self.file, &meta))
        };

        let mut command = std::process::Command::new("appimagetool");
        command.arg(work).arg(path);

        self.execute(command);

        Ok(())
    }
}

impl AppImage {
    const FILE_APP_RUN: &'static str = r#"#!/bin/bash

export APPDIR="$(dirname "$(readlink -f "$0")")"
export PATH="$APPDIR/usr/bin/:$PATH"
export LD_LIBRARY_PATH="$APPDIR/usr/lib:$PATH"
export XDG_DATA_DIRS="$APPDIR/usr/share/:/usr/share/:$XDG_DATA_DIRS"

"$APPDIR"/usr/bin/{name}
"#;

    fn file_app_run(meta: &Meta) -> String {
        let mut file = Self::FILE_APP_RUN.to_string();
        file = file.replace("{name}", &meta.name);

        file
    }
}
