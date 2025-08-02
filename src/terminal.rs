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

use crate::project::Project;

//================================================================

use clap::{Parser, Subcommand};
use std::path::PathBuf;

//================================================================

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Terminal {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the compile step on the project
    Compile {
        /// Path to project.
        #[arg(short, long)]
        path: PathBuf,
    },
    /// Run the package step on the project
    Package {
        /// Path to project.
        #[arg(short, long)]
        path: PathBuf,
    },
    /// Run the compile step, then the package step on the project
    CompileExport {
        /// Path to project.
        #[arg(short, long)]
        path: PathBuf,
    },
}

impl Terminal {
    pub fn run() -> anyhow::Result<bool> {
        let terminal = Terminal::parse();

        if let Some(command) = &terminal.command {
            match command {
                Command::Compile { path } => todo!(),
                Command::Package { path } => {
                    let mut project = Project::load(path.to_path_buf())?;

                    // package project.
                    project.package()?;

                    loop {
                        for package in &mut project.package {
                            package.poll_completion();
                        }

                        let complete = project
                            .package
                            .iter_mut()
                            .all(|package| package.success_or_failure());

                        if complete {
                            break;
                        }
                    }
                }
                Command::CompileExport { path } => todo!(),
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }
}
