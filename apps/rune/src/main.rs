use rune::AppSpec;
use rune_git_core::Browser;
mod ui;
use ui::view;
#[derive(Default)]
struct State {
    browser: Browser,
    split: bool,
}

fn main() {
    rune::run(AppSpec {
        setup: Some(|s: &mut State, wake| {
            s.browser.wake = wake;
            if let Some(path) = std::env::var_os("GIT_CLIENT_REPO") {
                s.browser.open(path.into());
            }
        }),
        poll: Some(|s: &mut State, _, tx| {
            let changed = s.browser.poll();
            if changed {
                tx.instant();
            }
            changed
        }),
        families: ("Inter", "ui-monospace"),
        min_size: Some((850.0, 550.0)),
        ..AppSpec::new("Arbor · Rune", (1440.0, 900.0), State::default, view)
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use rune::{Env, Runtime};
    use rune_git_core::Repository;
    use std::{
        process::Command,
        time::{Duration, Instant},
    };

    #[test]
    fn clicking_a_commit_loads_its_real_patch() {
        let dir = std::env::temp_dir().join(format!("rune-ui-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        for args in [
            vec!["init", "-b", "main"],
            vec!["config", "user.name", "UI Test"],
            vec!["config", "user.email", "ui@example.com"],
        ] {
            assert!(
                Command::new("git")
                    .arg("-C")
                    .arg(&dir)
                    .args(args)
                    .output()
                    .unwrap()
                    .status
                    .success()
            );
        }
        std::fs::write(dir.join("hello.txt"), "hello from Rune UI\n").unwrap();
        std::fs::write(dir.join("z-other.txt"), "other file contents\n").unwrap();
        for args in [vec!["add", "."], vec!["commit", "-m", "A real commit"]] {
            assert!(
                Command::new("git")
                    .arg("-C")
                    .arg(&dir)
                    .args(args)
                    .output()
                    .unwrap()
                    .status
                    .success()
            );
        }
        let repo = Repository::open(&dir).unwrap();
        let commits = repo.commits(0).unwrap();
        let oid = commits[0].oid.clone();
        let mut rt = Runtime::new(State::default(), view, (1200.0, 800.0), Env::new(true, 1.0));
        rt.poll = Some(|s, _, _| s.browser.poll());
        rt.frame(0.0);
        let (x, y) = rt.find("open").unwrap();
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.0);
        assert!(rt.file_dialog().is_some());
        rt.resolve_file_dialog(None).unwrap();
        rt.frame(0.0);
        assert!(
            rt.state.browser.repo.is_none(),
            "cancel leaves the welcome screen"
        );
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.0);
        rt.resolve_file_dialog(Some(dir.clone())).unwrap();
        rt.frame(0.0);
        let deadline = Instant::now() + Duration::from_secs(5);
        while rt.state.browser.busy && Instant::now() < deadline {
            rt.frame(0.01);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(rt.state.browser.commits.len(), 1);
        let (x, y) = rt.find(&oid).expect("commit row is mounted");
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.0);
        let deadline = Instant::now() + Duration::from_secs(5);
        while rt.state.browser.busy && Instant::now() < deadline {
            rt.frame(0.01);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert_eq!(rt.state.browser.detail.as_ref().unwrap().oid, oid);
        assert!(rt.state.browser.patch.contains("+hello from Rune UI"));
        let (x, y) = rt.find("file-1").expect("second file row is mounted");
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.0);
        let deadline = Instant::now() + Duration::from_secs(5);
        while rt.state.browser.busy && Instant::now() < deadline {
            rt.frame(0.01);
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(rt.state.browser.patch.contains("+other file contents"));
        assert!(!rt.state.browser.patch.contains("hello from Rune UI"));
        let (x, y) = rt.find("split").unwrap();
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.0);
        assert!(rt.state.split);
        let (x, y) = rt.find("unified").unwrap();
        rt.pointer_down(x, y);
        rt.pointer_up(x, y);
        rt.frame(0.0);
        assert!(!rt.state.split);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
