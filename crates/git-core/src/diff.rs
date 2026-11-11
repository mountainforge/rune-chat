#[derive(Clone, Debug, Default)]
pub struct DiffLine {
    pub old: String,
    pub new: String,
    pub text: String,
    pub kind: u8,
}
#[derive(Clone, Debug, Default)]
pub struct DiffPair {
    pub left: Option<DiffLine>,
    pub right: Option<DiffLine>,
    pub full: bool,
}
/// Decode hunk coordinates; file preambles are replaced by the file header in the UI.
pub fn lines(patch: &str) -> Vec<DiffLine> {
    let has_hunks = patch.lines().any(|l| l.starts_with("@@ "));
    let mut out = vec![];
    let (mut old, mut new) = (0, 0);
    let mut in_hunk = false;
    for line in patch.lines() {
        if line.starts_with("@@ ") {
            let p: Vec<_> = line.split_whitespace().collect();
            old = p
                .get(1)
                .and_then(|s| {
                    s.trim_start_matches('-')
                        .split(',')
                        .next()?
                        .parse::<usize>()
                        .ok()
                })
                .unwrap_or(0);
            new = p
                .get(2)
                .and_then(|s| {
                    s.trim_start_matches('+')
                        .split(',')
                        .next()?
                        .parse::<usize>()
                        .ok()
                })
                .unwrap_or(0);
            in_hunk = true;
            out.push(DiffLine {
                text: line.into(),
                kind: 3,
                ..Default::default()
            });
            continue;
        }
        if !in_hunk && has_hunks {
            continue;
        }
        let kind = if in_hunk {
            match line.as_bytes().first() {
                Some(b'+') => 1,
                Some(b'-') => 2,
                Some(b' ') => 0,
                _ => 4,
            }
        } else {
            4
        };
        let o = if kind == 0 || kind == 2 {
            let v = old.to_string();
            old += 1;
            v
        } else {
            String::new()
        };
        let n = if kind == 0 || kind == 1 {
            let v = new.to_string();
            new += 1;
            v
        } else {
            String::new()
        };
        out.push(DiffLine {
            old: o,
            new: n,
            text: if kind < 3 {
                line[1..].into()
            } else {
                line.into()
            },
            kind,
        });
    }
    out
}
pub fn pairs(lines: &[DiffLine]) -> Vec<DiffPair> {
    let mut out = vec![];
    let mut i = 0;
    while i < lines.len() {
        let l = &lines[i];
        if l.kind == 1 || l.kind == 2 {
            let (mut left, mut right) = (vec![], vec![]);
            while i < lines.len() && (lines[i].kind == 1 || lines[i].kind == 2) {
                if lines[i].kind == 2 {
                    left.push(lines[i].clone());
                } else {
                    right.push(lines[i].clone());
                }
                i += 1;
            }
            for n in 0..left.len().max(right.len()) {
                out.push(DiffPair {
                    left: left.get(n).cloned(),
                    right: right.get(n).cloned(),
                    full: false,
                });
            }
        } else {
            out.push(DiffPair {
                left: Some(l.clone()),
                right: if l.kind == 0 { Some(l.clone()) } else { None },
                full: l.kind >= 3,
            });
            i += 1;
        }
    }
    out
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn pairs_deletions_and_insertions_with_real_line_numbers() {
        let l = lines(
            "diff --git a/a b/a\n--- a/a\n+++ b/a\n@@ -10,2 +10,3 @@\n-old\n+new\n+extra\n same\n",
        );
        assert_eq!(l[1].old, "10");
        assert_eq!(l[2].new, "10");
        let p = pairs(&l);
        assert_eq!(p.len(), 4);
        assert_eq!(p[1].left.as_ref().unwrap().text, "old");
        assert_eq!(p[1].right.as_ref().unwrap().text, "new");
        assert!(p[2].left.is_none());
        assert_eq!(p[3].right.as_ref().unwrap().new, "12");
    }
    #[test]
    fn binary_notices_are_not_discarded() {
        assert!(
            lines("Binary files a/a and b/a differ")[0]
                .text
                .contains("Binary files")
        );
    }
}
