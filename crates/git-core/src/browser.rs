use crate::{Commit, Detail, GraphRow, PAGE_SIZE, Repository, graph};
use std::{
    path::PathBuf,
    sync::{
        Arc,
        mpsc::{self, Receiver},
    },
};

enum Update {
    Open(Result<(Repository, Vec<Commit>), String>),
    More(Result<Vec<Commit>, String>),
    Detail(Result<Detail, String>),
    File(Result<String, String>),
}

pub struct Browser {
    pub repo: Option<Repository>,
    pub commits: Vec<Commit>,
    pub detail: Option<Detail>,
    pub selected: Option<String>,
    pub error: String,
    pub graph: Vec<GraphRow>,
    pub graph_lanes: usize,
    pub selected_file: Option<usize>,
    pub patch: String,
    pub busy: bool,
    pub more: bool,
    rx: Option<Receiver<Update>>,
    pub wake: Arc<dyn Fn() + Send + Sync>,
}

impl Default for Browser {
    fn default() -> Self {
        Self {
            repo: None,
            commits: vec![],
            detail: None,
            selected: None,
            error: String::new(),
            graph: vec![],
            graph_lanes: 1,
            selected_file: None,
            patch: String::new(),
            busy: false,
            more: false,
            rx: None,
            wake: Arc::new(|| {}),
        }
    }
}

impl Browser {
    fn job(&mut self, f: impl FnOnce() -> Update + Send + 'static) {
        self.busy = true;
        self.error.clear();
        let (tx, rx) = mpsc::channel();
        self.rx = Some(rx);
        let wake = self.wake.clone();
        std::thread::spawn(move || {
            let _ = tx.send(f());
            wake();
        });
    }
    pub fn open(&mut self, path: PathBuf) {
        if self.busy {
            return;
        }
        self.job(move || {
            Update::Open(Repository::open(path).and_then(|r| r.commits(0).map(|c| (r, c))))
        });
    }
    pub fn refresh(&mut self) {
        if let Some(r) = &self.repo {
            self.open(r.path.clone());
        }
    }
    pub fn load_more(&mut self) {
        if self.busy || !self.more {
            return;
        }
        if let Some(repo) = self.repo.clone() {
            let offset = self.commits.len();
            self.job(move || Update::More(repo.commits(offset)));
        }
    }
    pub fn select(&mut self, oid: String) {
        if self.busy {
            return;
        }
        if let Some(repo) = self.repo.clone() {
            self.selected = Some(oid.clone());
            self.detail = None;
            self.selected_file = None;
            self.patch.clear();
            self.job(move || Update::Detail(repo.detail(&oid)));
        }
    }
    pub fn select_file(&mut self, index: usize) {
        if self.busy {
            return;
        }
        if let (Some(repo), Some(d)) = (self.repo.clone(), self.detail.clone()) {
            if let Some(file) = d.files.get(index).cloned() {
                self.selected_file = Some(index);
                self.patch.clear();
                self.job(move || Update::File(repo.file_patch(&d, &file)));
            }
        }
    }
    pub fn poll(&mut self) -> bool {
        let Some(update) = self.rx.as_ref().and_then(|rx| rx.try_recv().ok()) else {
            return false;
        };
        self.rx = None;
        self.busy = false;
        match update {
            Update::Open(Ok((r, c))) => {
                self.repo = Some(r);
                self.more = c.len() == PAGE_SIZE;
                self.commits = c;
                (self.graph, self.graph_lanes) = graph(&self.commits);
                self.detail = None;
                self.selected_file = None;
                self.patch.clear();
                self.selected = None;
                if let Some(first) = self.commits.first() {
                    self.select(first.oid.clone());
                }
            }
            Update::More(Ok(c)) => {
                self.more = c.len() == PAGE_SIZE;
                self.commits.extend(c);
                (self.graph, self.graph_lanes) = graph(&self.commits);
            }
            Update::Detail(Ok(d)) => {
                self.detail = Some(d);
                self.select_file(0);
            }
            Update::File(Ok(patch)) => self.patch = patch,
            Update::Open(Err(e))
            | Update::More(Err(e))
            | Update::Detail(Err(e))
            | Update::File(Err(e)) => self.error = e,
        }
        true
    }
}
