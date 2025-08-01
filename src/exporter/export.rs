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

use crate::project::Meta;

//================================================================

use eframe::egui::{self, Color32};
use std::{
    fmt::Display,
    process::Command,
    sync::mpsc::{Receiver, Sender, channel},
};

//================================================================

pub type EventTx = Sender<anyhow::Result<()>>;
pub type EventRx = Receiver<anyhow::Result<()>>;
pub type EventHandler = Option<(EventTx, EventRx)>;

#[typetag::serde(tag = "type")]
pub trait Export {
    fn draw_setup(&mut self, ui: &mut egui::Ui);
    fn draw_modal(&mut self, ui: &mut egui::Ui);
    fn get_enable(&self) -> bool;
    fn get_remove(&self) -> bool;
    fn get_status(&mut self) -> &mut ExportStatus;
    fn get_handler(&mut self) -> &mut EventHandler;
    fn run(&mut self, meta: Meta) -> anyhow::Result<()>;

    //================================================================

    /// set the current status.
    fn set_status(&mut self, status: ExportStatus) {
        *self.get_status() = status;
    }

    /// execute command.
    fn execute(&mut self, mut command: Command) {
        if self.get_handler().is_none() {
            let (tx, rx) = channel();

            *self.get_handler() = Some((tx, rx))
        }

        let (tx, _) = self.get_handler().as_ref().unwrap();

        let tx = tx.clone();

        std::thread::spawn(move || {
            let out = command.output().unwrap();

            if let Ok(stdout) = String::from_utf8(out.stdout.clone()) {
                println!("{stdout}");
            }

            if let Ok(stderr) = String::from_utf8(out.stderr.clone()) {
                println!("{stderr}");
            }

            let event = if out.clone().exit_ok().is_err() {
                Err(anyhow::Error::msg(String::from_utf8(out.stderr).unwrap()))
            } else {
                Ok(())
            };

            tx.send(event).unwrap();
        });
    }

    /// poll for completion.
    fn poll_completion(&mut self) {
        if let Some((_, rx)) = self.get_handler()
            && let Ok(event) = rx.try_recv()
        {
            match event {
                Ok(_) => self.set_status(ExportStatus::Success),
                Err(error) => self.set_status(ExportStatus::Failure(error.to_string())),
            }
        }
    }

    /// check if the exporter is no longer in progress.
    fn success_or_failure(&mut self) -> bool {
        !self.get_enable() || *self.get_status() != ExportStatus::InProgress
    }
}

pub fn format_tag(text: &str, tag: &str) -> String {
    if tag.is_empty() {
        text.to_string()
    } else {
        format!("{text} - ({tag})")
    }
}

pub fn format_tag_name(text: &str, tag: &str) -> String {
    if tag.is_empty() {
        text.to_string()
    } else {
        format!("{text}-{tag}")
    }
}

pub fn format_tag_present(tag: &str) -> String {
    if tag.is_empty() {
        "".to_string()
    } else {
        format!("_{tag}")
    }
}

//================================================================

#[derive(Default, PartialEq, Eq)]
pub enum ExportStatus {
    #[default]
    InProgress,
    Success,
    Failure(String),
}

impl ExportStatus {
    pub fn color(&self) -> Color32 {
        match self {
            ExportStatus::InProgress => Color32::LIGHT_BLUE,
            ExportStatus::Success => Color32::LIGHT_GREEN,
            ExportStatus::Failure(_) => Color32::LIGHT_RED,
        }
    }
}

impl Display for ExportStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportStatus::InProgress => f.write_str("In Progress"),
            ExportStatus::Success => f.write_str("Success"),
            ExportStatus::Failure(error) => f.write_str(&format!("Failure: {error}")),
        }
    }
}
