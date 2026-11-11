//! Read-only Git operations shared by the two Rust frontends.
use std::{
    path::{Path, PathBuf},
    process::Command,
};
mod browser;
pub mod diff;
pub mod presentation;
pub use browser::Browser;
mod graph;
pub use graph::{GRAPH_COLORS, GraphLine, GraphRow, graph};

pub const PAGE_SIZE: usize = 100;

#[derive(Clone, Debug, Default)]
pub struct Commit {
    pub oid: String,
    pub author: String,
    pub date: String,
    pub subject: String,
    pub parents: Vec<String>,
    pub refs: String,
}

#[derive(Clone, Debug, Default)]
pub struct Detail {
    pub oid: String,
    pub author: String,
    pub email: String,
    pub date: String,
    pub parents: String,
    pub message: String,
    pub files: Vec<ChangedFile>,
}

#[derive(Clone, Debug, Default)]
pub struct ChangedFile {
    pub status: String,
    pub path: String,
    pub old_path: String,
    pub additions: usize,
    pub deletions: usize,
    pub binary: bool,
}

#[derive(Clone, Debug)]
pub struct Repository {
    pub path: PathBuf,
}

impl Repository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let path = path.as_ref().canonicalize().map_err(|e| e.to_string())?;
        let repo = Self { path };
        repo.git(&["rev-parse", "--absolute-git-dir"])?;
        Ok(repo)
    }

    fn git(&self, args: &[&str]) -> Result<String, String> {
        let out = Command::new("git")
            .args([
                "--no-pager",
                "--literal-pathspecs",
                "--no-optional-locks",
                "-c",
                "color.ui=false",
                "-C",
            ])
            .arg(&self.path)
            .args(args)
            .env("GIT_TERMINAL_PROMPT", "0")
            .output()
            .map_err(|e| format!("Could not run Git: {e}"))?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_owned());
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }

    pub fn commits(&self, offset: usize) -> Result<Vec<Commit>, String> {
        let head = self.git(&["rev-parse", "--verify", "HEAD"]).is_ok();
        let out = self.git(&[
            "log",
            "--all",
            if head { "HEAD" } else { "--all" },
            "--topo-order",
            "--decorate=short",
            "-z",
            "--format=%H%x00%an%x00%aI%x00%s%x00%P%x00%D",
            &format!("--skip={offset}"),
            &format!("--max-count={PAGE_SIZE}"),
        ])?;
        let fields: Vec<_> = out.split_terminator('\0').collect();
        if fields.len() % 6 != 0 {
            return Err("Unexpected Git log output".into());
        }
        Ok(fields
            .chunks_exact(6)
            .map(|f| Commit {
                oid: f[0].into(),
                author: f[1].into(),
                date: f[2].into(),
                subject: f[3].into(),
                parents: f[4].split_whitespace().map(str::to_owned).collect(),
                refs: f[5].into(),
            })
            .collect())
    }

    pub fn detail(&self, oid: &str) -> Result<Detail, String> {
        if !matches!(oid.len(), 40 | 64) || !oid.bytes().all(|b| b.is_ascii_hexdigit()) {
            return Err("Invalid commit ID".into());
        }
        let out = self.git(&[
            "show",
            "--no-patch",
            "--format=%H%x00%an%x00%ae%x00%aI%x00%P%x00%B",
            oid,
            "--",
        ])?;
        let f: Vec<_> = out.splitn(6, '\0').collect();
        if f.len() != 6 {
            return Err("Unexpected Git commit output".into());
        }
        let mut args = vec![
            "diff-tree",
            "--root",
            "--no-commit-id",
            "-r",
            "-z",
            "--name-status",
            "--find-renames",
        ];
        if let Some(parent) = f[4].split_whitespace().next() {
            args.push(parent);
        }
        args.extend([oid, "--"]);
        let raw = self.git(&args)?;
        let fields: Vec<_> = raw.split_terminator('\0').collect();
        let mut files = vec![];
        let mut i = 0;
        while i < fields.len() {
            let status = fields[i];
            i += 1;
            let renamed = status.starts_with('R') || status.starts_with('C');
            let needed = if renamed { 2 } else { 1 };
            if i + needed > fields.len() {
                return Err("Unexpected Git file list".into());
            }
            let old_path = if renamed {
                fields[i].to_owned()
            } else {
                String::new()
            };
            let path = fields[i + needed - 1].to_owned();
            i += needed;
            files.push(ChangedFile {
                status: status.into(),
                path,
                old_path,
                additions: 0,
                deletions: 0,
                binary: false,
            });
        }
        // Numstat is NUL-delimited as well; a rename carries separate old/new paths.
        let mut stat_args = vec![
            "diff-tree",
            "--root",
            "--no-commit-id",
            "-r",
            "-z",
            "--numstat",
            "--find-renames",
        ];
        if let Some(parent) = f[4].split_whitespace().next() {
            stat_args.push(parent);
        }
        stat_args.extend([oid, "--"]);
        let stats = self.git(&stat_args)?;
        let fields: Vec<_> = stats.split_terminator('\0').collect();
        let mut i = 0;
        while i < fields.len() {
            let parts: Vec<_> = fields[i].splitn(3, '\t').collect();
            i += 1;
            if parts.len() != 3 {
                return Err("Unexpected Git statistics".into());
            }
            let path = if parts[2].is_empty() {
                if i + 2 > fields.len() {
                    return Err("Unexpected Git rename statistics".into());
                }
                let p = fields[i + 1];
                i += 2;
                p
            } else {
                parts[2]
            };
            if let Some(file) = files.iter_mut().find(|f| f.path == path) {
                file.binary = parts[0] == "-";
                file.additions = parts[0].parse().unwrap_or(0);
                file.deletions = parts[1].parse().unwrap_or(0);
            }
        }
        Ok(Detail {
            oid: f[0].into(),
            author: f[1].into(),
            email: f[2].into(),
            date: f[3].into(),
            parents: f[4].into(),
            message: f[5].trim_end().into(),
            files,
        })
    }
    pub fn file_patch(&self, detail: &Detail, file: &ChangedFile) -> Result<String, String> {
        if !matches!(detail.oid.len(), 40 | 64)
            || !detail.oid.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err("Invalid commit ID".into());
        }
        let mut args = vec![
            "show",
            "--format=",
            "--patch",
            "--root",
            "--diff-merges=first-parent",
            "--find-renames",
            "--no-ext-diff",
            "--no-textconv",
            &detail.oid,
            "--",
            &file.path,
        ];
        if !file.old_path.is_empty() {
            args.push(&file.old_path);
        }
        let mut patch = self.git(&args)?;
        if patch.len() > 2_000_000 {
            let mut end = 2_000_000;
            while !patch.is_char_boundary(end) {
                end -= 1;
            }
            patch.truncate(end);
            patch.push_str("\n[Display truncated at 2 MB]");
        }
        Ok(patch)
    }
}

#[cfg(test)]
mod tests;
