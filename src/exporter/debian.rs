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
    exporter::export::{EventHandler, Export},
    project::{CompileStatus, Meta},
};

//================================================================

use eframe::egui::{self, RichText};
use serde::{Deserialize, Serialize};

//================================================================

#[derive(Default, Serialize, Deserialize)]
pub struct Debian {
    pub binary: Option<String>,
    pub script_prior: Option<String>,
    pub script_after: Option<String>,
    pub export: bool,
    pub x86_64: bool,
    pub architecture: String,
    #[serde(skip)]
    pub status: CompileStatus,
    #[serde(skip)]
    pub handler: EventHandler,
}

#[typetag::serde]
impl Export for Debian {
    fn draw_setup(&mut self, ui: &mut egui::Ui) -> bool {
        ui.collapsing("Debian Package (.deb)", |ui| {
            ui.checkbox(&mut self.export, "Export Package");

            ui.add_enabled_ui(self.export, |ui| {
                App::pick_file(ui, "Binary", &mut self.binary);
                App::pick_file(ui, "Prior-Installation Script", &mut self.script_prior);
                App::pick_file(ui, "After-Installation Script", &mut self.script_after);
                ui.checkbox(&mut self.x86_64, "64-Bit Binary")
                //self.desktop.draw(ui);
            });

            ui.button("Remove").clicked()
        });

        false
    }

    fn draw_modal(&mut self, ui: &mut egui::Ui) {
        if self.export {
            ui.horizontal(|ui| {
                ui.label(".deb package status: ");
                ui.label(RichText::new(format!("{}", self.status)).color(self.status.color()));

                if self.status == CompileStatus::InProgress {
                    ui.spinner();
                }
            });
        }
    }

    fn get_export(&self) -> bool {
        self.export
    }

    fn get_status(&mut self) -> &mut CompileStatus {
        &mut self.status
    }

    fn get_handler(&mut self) -> &mut EventHandler {
        &mut self.handler
    }

    fn compile(&mut self, meta: Meta) -> anyhow::Result<()> {
        if !self.export {
            return Ok(());
        }

        self.status = CompileStatus::InProgress;

        if meta.name.is_empty() {
            return Err(anyhow::Error::msg("Debian: Project name cannot be empty."));
        }

        if meta.version.is_empty() {
            return Err(anyhow::Error::msg(
                "Debian: Project version cannot be empty.",
            ));
        }

        let work = format!("test/boondle_debian/{}_{}_all", meta.name, meta.version);
        let debian = format!("{work}/DEBIAN");
        let usr = format!("{work}/usr");

        if std::fs::exists("test/boondle_debian")? {
            std::fs::remove_dir_all("test/boondle_debian")?;
        }

        // create work folder.
        std::fs::create_dir_all(&work)?;

        //================================================================

        // create DEBIAN folder.
        std::fs::create_dir(&debian)?;

        // write control file.
        std::fs::write(format!("{debian}/control"), Self::file_control(&meta))?;

        // copy prior-install script, if present.
        if let Some(path) = &self.script_prior {
            std::fs::copy(path, format!("{debian}/preinst"))?;
        }

        // copy after-install script.
        if let Some(path) = &self.script_after {
            std::fs::copy(path, format!("{debian}/postinst"))?;
        }

        //================================================================

        // create usr folder.
        std::fs::create_dir(&usr)?;

        // create binary folder.
        std::fs::create_dir_all(format!("{usr}/bin"))?;

        // copy binary, if present.
        if let Some(path) = &self.binary {
            std::fs::copy(path, format!("{usr}/bin/{}", meta.name))?;
        }

        // create application folder.
        std::fs::create_dir_all(format!("{usr}/share/applications"))?;

        // write .desktop file.
        std::fs::write(
            format!("{usr}/share/applications/{}.desktop", meta.name),
            meta.create_desktop_file(false),
        )?;

        // create icon folder.
        std::fs::create_dir_all(format!("{usr}/share/icons"))?;

        // copy icon file, if present.
        if let Some(path) = &meta.icon {
            std::fs::copy(path, format!("{usr}/share/icons/{}-icon", meta.name))?;
        }

        //================================================================

        let path = format!("test/{}_{}.deb", meta.name, meta.version);

        let mut command = std::process::Command::new("dpkg-deb");
        command.arg("--build").arg(work).arg(path);

        self.execute(command);

        Ok(())
    }
}

impl Debian {
    const FILE_CONTROL: &'static str = r#"Package: {name}
Version: {version}
Architecture: all
Essential: no
Priority: optional
Depends:
Maintainer: {from}
Description: {info}
"#;

    fn file_control(meta: &Meta) -> String {
        let mut file = Self::FILE_CONTROL.to_string();
        file = file.replace("{name}", &meta.name);
        file = file.replace("{info}", &meta.info);
        file = file.replace("{from}", &meta.from);
        file = file.replace("{version}", &meta.version);

        file
    }
}
