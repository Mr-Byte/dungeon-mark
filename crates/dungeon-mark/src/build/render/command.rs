use shlex::Shlex;
use std::{
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

use super::Renderer;
use crate::error::Result;

pub struct CommandRenderer {
    name: String,
    command: String,
}

impl CommandRenderer {
    pub fn new(name: String, command: String) -> Self {
        Self { name, command }
    }
}

impl CommandRenderer {
    fn build_command(&self, root: &Path) -> Result<Command> {
        let mut parts = Shlex::new(&self.command);
        let Some(bin) = parts.next() else {
            anyhow::bail!("Provided command string was empty");
        };

        // NOTE: Get the path to the binary.
        let bin = PathBuf::from(bin);
        let bin = if bin.components().count() == 1 {
            // NOTE: Search for the binary in PATH.
            bin
        } else {
            // NOTE: Search for the binary relative to the project root.
            root.join(bin)
        };

        let mut command = Command::new(bin);
        command.args(parts);

        Ok(command)
    }
}

impl Renderer for CommandRenderer {
    fn name(&self) -> &str {
        &self.name
    }

    fn render(&self, ctx: super::RenderContext) -> anyhow::Result<()> {
        let mut process = self
            .build_command(&ctx.root)?
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut stdin = process.stdin.take().expect("Child process has stdin");
        // TODO: Docs said this should be done on a separate thread to prevent a deadlock?
        if let Err(err) = serde_json::to_writer(&mut stdin, &ctx) {
            dbg!(err);
            // TODO: Emit warnings about errors?
        }

        // NOTE: Explicitly drop stdin to close it.
        drop(stdin);

        let status = process.wait()?;

        if !status.success() {
            anyhow::bail!("Renderer {} failed ({}).", self.name, status);
        }

        // TODO: Handle errors

        Ok(())
    }
}
