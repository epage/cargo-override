use crate::{cli, CargoInvocation, Cli};

use anyhow::bail;
use camino::Utf8PathBuf;
use cargo_util_schemas::core::GitReference;
use url::Url;

pub struct Context {
    pub cargo: Cargo,

    pub registry_hint: Option<String>,

    pub manifest_path: Option<Utf8PathBuf>,

    pub mode: Mode,
}

#[derive(Copy, Clone)]
pub struct Cargo {
    pub locked: bool,
    pub offline: bool,
}

pub enum Mode {
    Path(Utf8PathBuf),
    Git { url: Url, reference: GitReference },
}

impl TryFrom<Cli> for Context {
    type Error = anyhow::Error;

    fn try_from(
        Cli {
            command:
                CargoInvocation::Override(cli::Override {
                    locked,
                    offline,
                    frozen,
                    registry,
                    manifest_path,
                    source: cli::Source { path, git },
                    git: cli::Git { branch, tag, rev },
                }),
        }: Cli,
    ) -> Result<Self, Self::Error> {
        // `--frozen` implies `--locked` and `--offline`
        let [locked, offline] = [locked, offline].map(|f| f || frozen);

        let cargo = Cargo { locked, offline };

        let mode = match (git, path) {
            (Some(git), None) => Mode::Git {
                url: git,
                reference: {
                    match (branch, tag, rev) {
                        (None, None, None) => GitReference::DefaultBranch,
                        (Some(branch), None, None) => GitReference::Branch(branch),
                        (None, Some(tag), None) => GitReference::Tag(tag),
                        (None, None, Some(rev)) => GitReference::Rev(rev),
                        _ => bail!("multiple git identifiers used. Only use one of `--branch`, `--tag` or `--rev`")

                    }
                },
            },
            (None, Some(path)) => Mode::Path(path),
            (Some(_), Some(_)) => {
                bail!("`--git` can not bot set at the same time as `--path`")
            }
            (None, None) => {
                bail!("specify a package to patch with using `--path` or `--git`")
            }
        };

        Ok(Self {
            cargo,

            registry_hint: registry,

            manifest_path,

            mode,
        })
    }
}
