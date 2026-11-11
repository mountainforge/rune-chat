use super::*;
use std::sync::atomic::{AtomicUsize, Ordering};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
impl Fixture {
    fn new() -> Self {
        let f = Self(std::env::temp_dir().join(format!(
            "git client test {} {}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        std::fs::create_dir_all(&f.0).unwrap();
        f.git(&["init", "-b", "main"]);
        f.git(&["config", "user.name", "Test Author"]);
        f.git(&["config", "user.email", "test@example.com"]);
        f
    }
    fn git(&self, args: &[&str]) {
        assert!(
            Command::new("git")
                .arg("-C")
                .arg(&self.0)
                .args(args)
                .output()
                .unwrap()
                .status
                .success()
        );
    }
    fn commit(&self, message: &str) {
        self.git(&["add", "."]);
        self.git(&["commit", "--allow-empty", "-m", message]);
    }
}
#[test]
fn empty_invalid_and_revision_validation() {
    let f = Fixture::new();
    let r = Repository::open(&f.0).unwrap();
    assert!(r.commits(0).unwrap().is_empty());
    assert!(r.detail("--help").is_err());
    assert!(Repository::open(f.0.join("missing")).is_err());
}
#[test]
fn history_root_patch_unicode_and_paging() {
    let f = Fixture::new();
    std::fs::write(f.0.join("hello world.txt"), "héllo\n").unwrap();
    f.commit("First 🦀\n\nBody with details");
    let r = Repository::open(&f.0).unwrap();
    let c = r.commits(0).unwrap();
    assert_eq!(c.len(), 1);
    assert_eq!(c[0].subject, "First 🦀");
    let d = r.detail(&c[0].oid).unwrap();
    assert_eq!(d.author, "Test Author");
    assert!(d.parents.is_empty());
    assert_eq!(d.files[0].additions, 1);
    assert_eq!(d.files[0].deletions, 0);
    assert!(d.message.contains("Body with details"));
    assert!(r.file_patch(&d, &d.files[0]).unwrap().contains("+héllo"));
    for _ in 0..100 {
        f.commit("Later");
    }
    assert_eq!(r.commits(0).unwrap().len(), PAGE_SIZE);
    assert_eq!(r.commits(100).unwrap()[0].oid, c[0].oid);
}
#[test]
fn merge_and_bare_repo() {
    let f = Fixture::new();
    f.commit("Root");
    f.git(&["checkout", "-b", "feature"]);
    std::fs::write(f.0.join("feature.txt"), "feature\n").unwrap();
    f.commit("Feature");
    f.git(&["checkout", "main"]);
    f.commit("Main");
    f.git(&["merge", "--no-ff", "feature", "-m", "Merge"]);
    let r = Repository::open(&f.0).unwrap();
    let c = r.commits(0).unwrap();
    let d = r.detail(&c[0].oid).unwrap();
    assert_eq!(d.parents.split_whitespace().count(), 2);
    assert!(
        r.file_patch(&d, &d.files[0])
            .unwrap()
            .contains("feature.txt")
    );
    assert_eq!(
        Repository::open(f.0.join(".git"))
            .unwrap()
            .commits(0)
            .unwrap()
            .len(),
        c.len()
    );
}

#[test]
fn unmerged_branches_graph_and_file_scoping() {
    let f = Fixture::new();
    std::fs::write(f.0.join("a.txt"), "one\n").unwrap();
    std::fs::write(f.0.join("b.txt"), "two\n").unwrap();
    f.commit("Root");
    f.git(&["checkout", "-b", "unmerged"]);
    f.commit("Only on feature");
    f.git(&["checkout", "main"]);
    f.git(&["mv", "a.txt", "renamed.txt"]);
    std::fs::write(f.0.join("b.txt"), "changed\n").unwrap();
    std::fs::write(f.0.join(":(glob)*"), "literal filename\n").unwrap();
    f.commit("Rename and modify");
    let r = Repository::open(&f.0).unwrap();
    let commits = r.commits(0).unwrap();
    assert!(
        commits
            .iter()
            .any(|c| c.subject == "Only on feature" && c.refs.contains("unmerged"))
    );
    let (rows, width) = graph(&commits);
    assert_eq!(rows.len(), commits.len());
    assert!(width >= 2);
    let oid = &commits
        .iter()
        .find(|c| c.subject == "Rename and modify")
        .unwrap()
        .oid;
    let d = r.detail(oid).unwrap();
    assert_eq!(d.files.len(), 3);
    let renamed = d.files.iter().find(|f| f.status.starts_with('R')).unwrap();
    assert_eq!(renamed.old_path, "a.txt");
    assert_eq!(renamed.path, "renamed.txt");
    let b = d.files.iter().find(|f| f.path == "b.txt").unwrap();
    let patch = r.file_patch(&d, b).unwrap();
    assert_eq!(b.additions, 1);
    assert_eq!(b.deletions, 1);
    assert!(patch.contains("+changed"));
    assert!(!patch.contains("renamed.txt"));
    let literal = d.files.iter().find(|f| f.path == ":(glob)*").unwrap();
    assert!(
        r.file_patch(&d, literal)
            .unwrap()
            .contains("+literal filename")
    );
}
