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
    project::{CompileStatus, Project},
};

//================================================================

use eframe::egui;
use serde::{Deserialize, Serialize};

//================================================================

#[derive(Default, Serialize, Deserialize)]
pub struct AppImage {
    pub script: Option<String>,
    pub desktop: crate::exporter::debian::Desktop,
    pub export: bool,
    #[serde(skip)]
    pub status: CompileStatus,
}

impl AppImage {
    const FILE_APP_RUN: &'static str = r#"#!/bin/bash

export APPDIR="$(dirname "$(readlink -f "$0")")"
export PATH="$APPDIR/usr/bin/:$PATH"
export LD_LIBRARY_PATH="$APPDIR/usr/lib:$PATH"
export XDG_DATA_DIRS="$APPDIR/usr/share/:/usr/share/:$XDG_DATA_DIRS"

"$APPDIR"/usr/bin/{name}
"#;

    pub fn draw(&mut self, ui: &mut egui::Ui) {
        ui.collapsing("AppImage Package (.AppImage)", |ui| {
            App::pick_file(ui, "After-Installation Script", &mut self.script);

            self.desktop.draw(ui);

            ui.checkbox(&mut self.export, "Export Package");
        });
    }

    pub fn export(project: &mut Project) -> anyhow::Result<()> {
        if !project.app_image.export {
            return Ok(());
        }

        project.app_image.status = CompileStatus::InProgress;

        if project.name.is_empty() {
            return Err(anyhow::Error::msg(
                "AppImage: Project name cannot be empty.",
            ));
        }

        let work = format!("test/boondle_app_image/{}.AppDir", project.name);
        let usr = format!("{work}/usr");

        if std::fs::exists("test/boondle_app_image")? {
            std::fs::remove_dir_all("test/boondle_app_image")?;
        }

        // create work folder.
        std::fs::create_dir_all(&work)?;

        //================================================================

        // copy after-install script.
        if let Some(path) = &project.app_image.script {
            std::fs::copy(path, format!("{work}/AppRun"))?;
        } else {
            // write AppRun file.
            std::fs::write(format!("{work}/AppRun"), Self::file_app_run(project))?;
        }

        std::process::Command::new("chmod")
            .arg("a+x")
            .arg(format!("{work}/AppRun"))
            .output()?;

        // write .desktop file.
        std::fs::write(
            format!("{work}/{}.desktop", project.name),
            project.app_image.desktop.create_file(project, true),
        )?;

        //================================================================

        // create binary folder.
        std::fs::create_dir_all(format!("{usr}/bin"))?;

        // copy binary, if present.
        if let Some(path) = &project.path_binary {
            std::fs::copy(path, format!("{usr}/bin/{}", project.name))?;
        }

        // copy icon file, if present.
        if let Some(path) = &project.icon {
            // appimagetool won't work if we don't have an extension at the end...
            std::fs::copy(path, format!("{work}/{}-icon.png", project.name))?;
        }

        //================================================================

        let path = format!("test/{}_{}.AppImage", project.name, project.version);
        let tx = project.event_tx.clone().unwrap();

        std::thread::spawn(move || {
            let out = std::process::Command::new("appimagetool")
                .arg(work)
                .arg(path)
                .output()
                .unwrap();

            let event = if out.clone().exit_ok().is_err() {
                crate::project::Event::AppImage(Err(anyhow::Error::msg(
                    String::from_utf8(out.stderr).unwrap(),
                )))
            } else {
                crate::project::Event::AppImage(Ok(()))
            };

            tx.send(event).unwrap();
        });

        Ok(())
    }

    fn file_app_run(project: &Project) -> String {
        let mut file = Self::FILE_APP_RUN.to_string();
        file = file.replace("{name}", &project.name);

        file
    }
}
