use super::State;
use rune::tok::{TextStyle, type_};
use rune::{Cx, El, FileDialogOptions, GrowthSpec, ScrollSpec, Shape, canvas, col, row, text};
use rune_git_core::{
    GRAPH_COLORS, GraphRow,
    diff::{DiffLine, lines, pairs},
    presentation::{avatar_color, file_parts, head_label, initials},
};
fn color(c: u32) -> rune::Color {
    rune::color::hex(&format!("#{c:06x}")).unwrap()
}
const INK: u32 = 0x191b20;
const MUTED: u32 = 0x525865;
const FAINT: u32 = 0x5f6572;
const LINE: u32 = 0xe4e1d9;
const RAISED: u32 = 0xf7f6f2;
const TEAL: u32 = 0x06748f;
const SELECT: u32 = 0xebf4f6;
fn label(s: impl Into<String>, size: f32, c: u32) -> El {
    text(s).font(TextStyle::new(size, 400.0)).color(color(c))
}
fn button(s: &str, on: bool) -> El {
    row()
        .padding(6.0)
        .radius(7.0)
        .fill(color(if on { 0xffffff } else { RAISED }))
        .border(1.0, color(if on { 0xd5d1c7 } else { LINE }))
        .role("button")
        .label(s)
        .focusable(true)
        .child(label(s, 12.0, if on { INK } else { MUTED }))
}
fn avatar(name: &str) -> El {
    row()
        .size(20.0, 20.0)
        .radius(10.0)
        .center()
        .fill(color(avatar_color(name)))
        .child(label(initials(name), 9.0, INK))
}
fn logo() -> El {
    let mut shape = Shape::new()
        .stroke(vec![[10.0, 7.0], [10.0, 23.0]], 2.2, color(0x9a5a00))
        .stroke(
            vec![[10.0, 19.0], [11.0, 15.0], [18.0, 13.0], [20.0, 9.0]],
            2.2,
            color(TEAL),
        );
    for (x, y, c) in [(10.0, 22.5, 0x9a5a00), (20.0, 9.0, TEAL)] {
        shape = shape.fill(
            (0..20)
                .map(|i| {
                    let a = i as f32 * std::f32::consts::TAU / 20.0;
                    [x + 2.6 * a.cos(), y + 2.6 * a.sin()]
                })
                .collect::<Vec<_>>(),
            color(c),
        );
    }
    canvas(shape)
        .size(30.0, 30.0)
        .fill(color(RAISED))
        .radius(8.0)
        .border(1.0, color(0xd5d1c7))
}
fn graph(g: &GraphRow, w: f32, selected: bool) -> El {
    let mut shape = Shape::new();
    for l in &g.lines {
        shape = shape.stroke(l.points(30.0), 1.8, color(GRAPH_COLORS[l.color % 6]));
    }
    let x = 14.0 + g.lane as f32 * 18.0;
    for (r, c) in [
        (
            if selected { 8.0 } else { 0.0 },
            color(GRAPH_COLORS[g.lane % 6]).alpha(0.18),
        ),
        (4.0, color(GRAPH_COLORS[g.lane % 6])),
    ] {
        if r > 0.0 {
            shape = shape.fill(
                (0..20)
                    .map(|i| {
                        let a = i as f32 * std::f32::consts::TAU / 20.0;
                        [x + r * a.cos(), 15.0 + r * a.sin()]
                    })
                    .collect::<Vec<_>>(),
                c,
            );
        }
    }
    canvas(shape).size(w, 30.0).shrink()
}
fn stat_color(status: &str) -> u32 {
    match status.as_bytes().first() {
        Some(b'A') => 0x1b6e2d,
        Some(b'D') => 0xae2016,
        _ => 0x9a5a00,
    }
}
fn badge(s: &str, c: u32, on: bool) -> El {
    label(s, 11.0, if on { 0xffffff } else { c })
        .padding(4.0)
        .radius(6.0)
        .fill(if on { color(c) } else { color(c).alpha(0.08) })
        .border(1.0, color(c).alpha(0.3))
}
fn diff_line(l: &DiffLine, split: bool, right: bool) -> El {
    let (fg, bg) = match l.kind {
        1 => (0x1b6e2d, 0xeaf6ec),
        2 => (0xae2016, 0xfcedeb),
        3 => (0x6a3fc7, 0xf4f0fc),
        _ => (INK, 0xffffff),
    };
    if l.kind >= 3 {
        return label(&l.text, 12.0, fg)
            .font(type_::MONO_CAPTION.mono())
            .h(21.0)
            .min_w(1000.0)
            .fill(color(bg));
    }
    let mut r = row().h(21.0).fill(color(bg));
    if !split {
        r = r
            .child(
                label(&l.old, 11.0, FAINT)
                    .font(type_::MONO_CAPTION.mono())
                    .w(40.0),
            )
            .child(
                label(&l.new, 11.0, FAINT)
                    .font(type_::MONO_CAPTION.mono())
                    .w(40.0),
            );
    } else {
        r = r.child(
            label(if right { &l.new } else { &l.old }, 11.0, FAINT)
                .font(type_::MONO_CAPTION.mono())
                .w(40.0),
        );
    }
    r.child(
        label(
            match l.kind {
                1 => "+",
                2 => "−",
                _ => " ",
            },
            12.0,
            fg,
        )
        .w(18.0),
    )
    .child(
        label(&l.text, 12.0, fg)
            .font(type_::MONO_CAPTION.mono())
            .min_w(if split { 440.0 } else { 900.0 }),
    )
}
pub(super) fn view(s: &State, _: &mut Cx<State>) -> El {
    let b = &s.browser;
    let gw = (b.graph_lanes as f32 * 18.0 + 28.0).max(76.0);
    let path = b
        .repo
        .as_ref()
        .map(|r| r.path.display().to_string())
        .unwrap_or_else(|| "Open a local repository".into());
    let name = b
        .repo
        .as_ref()
        .and_then(|r| r.path.file_name())
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Arbor".into());
    let toolbar = row()
        .h(60.0)
        .padding(12.0)
        .gap(12.0)
        .fill(color(0xffffff))
        .border_sides([0.0, 0.0, 1.0, 0.0], color(LINE))
        .child(logo())
        .child(
            col()
                .w(270.0)
                .min_w(0.0)
                .gap(3.0)
                .child(
                    label(name, 15.0, INK)
                        .font(TextStyle::new(15.0, 600.0))
                        .truncate(),
                )
                .child(label(path, 11.0, FAINT).truncate()),
        )
        .child(badge(&head_label(&b.commits), TEAL, false))
        .child(row().grow(1.0))
        .child(
            button("Open repository", false)
                .key("open")
                .on_press_tx(|s: &mut State, tx| {
                    if s.browser.busy {
                        return;
                    }
                    tx.open_folder(
                        FileDialogOptions::default(),
                        |s: &mut State, r, _| match r {
                            Ok(Some(p)) => s.browser.open(p),
                            Ok(None) => {}
                            Err(e) => s.browser.error = e.to_string(),
                        },
                    );
                }),
        )
        .child(
            button("Refresh", false)
                .key("refresh")
                .on_press(|s: &mut State| s.browser.refresh()),
        );
    let headings = row()
        .h(28.0)
        .gap(8.0)
        .fill(color(0xffffff))
        .child(label("GRAPH", 10.0, FAINT).w(gw))
        .child(label("DESCRIPTION", 10.0, FAINT).grow(1.0))
        .child(label("AUTHOR", 10.0, FAINT).w(156.0))
        .child(label("DATE", 10.0, FAINT).w(104.0))
        .child(label("COMMIT", 10.0, FAINT).w(80.0));
    let rows = col().growth(GrowthSpec::default()).virtual_rows(
        b.commits.len(),
        30.0,
        |s: &State, i| s.browser.commits[i].oid.as_str().into(),
        move |s: &State, i| {
            let b = &s.browser;
            let c = &b.commits[i];
            let oid = c.oid.clone();
            let on = b.selected.as_deref() == Some(&c.oid);
            let mut desc = row().grow(1.0).min_w(0.0).gap(6.0);
            for rf in c.refs.split(", ").filter(|r| !r.is_empty()).take(3) {
                let head = rf.starts_with("HEAD -> ");
                let txt = rf.strip_prefix("HEAD -> ").unwrap_or(rf);
                desc = desc.child(badge(txt, GRAPH_COLORS[b.graph[i].lane % 6], head).max_w(210.0));
            }
            desc = desc.child(
                label(
                    &c.subject,
                    13.0,
                    if c.parents.len() > 1 { MUTED } else { INK },
                )
                .grow(1.0)
                .min_w(0.0)
                .truncate(),
            );
            row()
                .h(30.0)
                .w_full()
                .gap(8.0)
                .role("button")
                .label(&c.subject)
                .focusable(true)
                .fill(color(if on { SELECT } else { 0xffffff }))
                .border(
                    1.0,
                    if on {
                        color(TEAL).alpha(0.35)
                    } else {
                        rune::Color::TRANSPARENT
                    },
                )
                .child(graph(&b.graph[i], gw, on))
                .child(desc)
                .child(
                    row().w(156.0).gap(8.0).child(avatar(&c.author)).child(
                        label(&c.author, 12.0, MUTED)
                            .grow(1.0)
                            .min_w(0.0)
                            .truncate(),
                    ),
                )
                .child(label(&c.date[..10], 12.0, FAINT).w(104.0))
                .child(
                    label(&c.oid[..8], 12.0, FAINT)
                        .font(type_::MONO_CAPTION.mono())
                        .w(80.0),
                )
                .on_press(move |s: &mut State| s.browser.select(oid.clone()))
        },
    );
    let mut history = col()
        .grow(0.95)
        .min_h(0.0)
        .fill(color(0xffffff))
        .child(
            row()
                .h(40.0)
                .padding(12.0)
                .gap(18.0)
                .child(label("History", 13.0, INK).font(TextStyle::new(13.0, 600.0)))
                .child(label(
                    format!("All branches · {} commits", b.commits.len()),
                    12.0,
                    FAINT,
                )),
        )
        .child(headings)
        .child(
            col()
                .key("history")
                .grow(1.0)
                .min_h(0.0)
                .scroll(ScrollSpec {
                    stick_bottom: false,
                })
                .child(rows),
        );
    if b.more {
        history = history.child(
            row().h(28.0).padding(2.0).child(
                button("Load more commits", false)
                    .key("more")
                    .on_press(|s: &mut State| s.browser.load_more()),
            ),
        );
    }
    if b.commits.is_empty() && !b.busy {
        history = history.child(
            label(
                "Open a repository to explore its branches and commits.",
                13.0,
                FAINT,
            )
            .padding(18.0),
        );
    }
    let mut bottom = row().grow(1.0).min_h(0.0).fill(color(0xffffff));
    if let Some(d) = &b.detail {
        let mut summary = col()
            .w_full()
            .gap(9.0)
            .padding(16.0)
            .border_sides([0.0, 0.0, 1.0, 0.0], color(LINE));
        let (title, body) = d.message.split_once('\n').unwrap_or((&d.message, ""));
        summary = summary.child(
            label(title, 15.0, INK)
                .font(TextStyle::new(15.0, 600.0))
                .wrap(),
        );
        if !body.trim().is_empty() {
            summary = summary.child(label(body.trim(), 12.0, MUTED).wrap());
        }
        summary = summary.child(
            row()
                .gap(8.0)
                .child(avatar(&d.author))
                .child(label(&d.author, 12.0, INK))
                .child(label(&d.date[..10], 11.0, FAINT)),
        );
        let mut chips = row()
            .gap(5.0)
            .child(badge(&d.oid[..8], MUTED, false).label(&d.oid));
        if d.parents.is_empty() {
            chips = chips.child(label("Root commit", 11.0, FAINT));
        } else {
            chips = chips.child(label("Parent", 11.0, FAINT));
            for p in d.parents.split_whitespace() {
                let oid = p.to_owned();
                chips = chips.child(
                    button(&p[..8], false)
                        .label(p)
                        .on_press(move |s: &mut State| s.browser.select(oid.clone())),
                );
            }
        }
        summary = summary.child(chips);
        let totals = (
            d.files.iter().map(|f| f.additions).sum::<usize>(),
            d.files.iter().map(|f| f.deletions).sum::<usize>(),
        );
        let sidebar = col()
            .w(340.0)
            .h_full()
            .min_h(0.0)
            .border_sides([0.0, 1.0, 0.0, 0.0], color(LINE))
            .child(
                col()
                    .max_h(230.0)
                    .w_full()
                    .scroll(ScrollSpec {
                        stick_bottom: false,
                    })
                    .child(summary),
            )
            .child(
                row()
                    .h(32.0)
                    .padding(8.0)
                    .gap(8.0)
                    .child(label(format!("{} FILES CHANGED", d.files.len()), 10.0, FAINT).grow(1.0))
                    .child(label(format!("+{}", totals.0), 11.0, 0x1b6e2d))
                    .child(label(format!("−{}", totals.1), 11.0, 0xae2016)),
            )
            .child(
                col()
                    .key(format!("files-{}", d.oid))
                    .grow(1.0)
                    .min_h(0.0)
                    .scroll(ScrollSpec {
                        stick_bottom: false,
                    })
                    .child(col().growth(GrowthSpec::default()).virtual_rows(
                        d.files.len(),
                        44.0,
                        |_: &State, i| format!("file-{i}").into(),
                        |s: &State, i| {
                            let f = &s.browser.detail.as_ref().unwrap().files[i];
                            let (dir, name) = file_parts(&f.path);
                            let on = s.browser.selected_file == Some(i);
                            row()
                                .h(44.0)
                                .w_full()
                                .padding(6.0)
                                .gap(8.0)
                                .radius(8.0)
                                .role("button")
                                .label(&f.path)
                                .focusable(true)
                                .fill(color(if on { SELECT } else { 0xffffff }))
                                .border(
                                    1.0,
                                    if on {
                                        color(TEAL).alpha(0.35)
                                    } else {
                                        rune::Color::TRANSPARENT
                                    },
                                )
                                .child(badge(&f.status[..1], stat_color(&f.status), false))
                                .child(
                                    col()
                                        .grow(1.0)
                                        .min_w(0.0)
                                        .gap(2.0)
                                        .child(label(name, 12.0, INK).truncate())
                                        .child(
                                            label(
                                                if dir.is_empty() {
                                                    "Repository root"
                                                } else {
                                                    dir
                                                },
                                                11.0,
                                                FAINT,
                                            )
                                            .truncate(),
                                        ),
                                )
                                .child(label(
                                    if f.binary {
                                        "Binary".into()
                                    } else {
                                        format!("+{} −{}", f.additions, f.deletions)
                                    },
                                    11.0,
                                    MUTED,
                                ))
                                .on_press(move |s: &mut State| s.browser.select_file(i))
                        },
                    )),
            );
        let file = b.selected_file.and_then(|i| d.files.get(i));
        let mut head = row()
            .h(52.0)
            .padding(10.0)
            .gap(8.0)
            .border_sides([0.0, 0.0, 1.0, 0.0], color(LINE));
        if let Some(f) = file {
            let (dir, name) = file_parts(&f.path);
            head = head
                .child(badge(&f.status[..1], stat_color(&f.status), false))
                .child(label(dir, 12.0, FAINT).truncate())
                .child(label(name, 13.0, INK).font(type_::BODY.mono()))
                .child(row().grow(1.0))
                .child(label(
                    format!("+{} −{}", f.additions, f.deletions),
                    12.0,
                    MUTED,
                ));
        }
        head = head.child(
            row()
                .padding(2.0)
                .radius(8.0)
                .fill(color(0xf0eee8))
                .child(
                    button("Unified", !s.split)
                        .key("unified")
                        .on_press(|s: &mut State| s.split = false),
                )
                .child(
                    button("Split", s.split)
                        .key("split")
                        .on_press(|s: &mut State| s.split = true),
                ),
        );
        let decoded = std::sync::Arc::new(lines(&b.patch));
        let paired = std::sync::Arc::new(pairs(&decoded));
        let count = if s.split { paired.len() } else { decoded.len() };
        let split = s.split;
        let code = col()
            .key(format!("patch-{}-{:?}-{split}", d.oid, b.selected_file))
            .grow(1.0)
            .min_h(0.0)
            .scroll(ScrollSpec {
                stick_bottom: false,
            })
            .scroll_xy()
            .child(col().growth(GrowthSpec::default()).virtual_rows(
                count,
                21.0,
                |_: &State, i| format!("line-{i}").into(),
                move |_: &State, i| {
                    if !split {
                        return diff_line(&decoded[i], false, false);
                    }
                    let p = &paired[i];
                    if p.full {
                        return diff_line(p.left.as_ref().unwrap(), false, false);
                    }
                    let mut r = row().h(21.0);
                    for (right, l) in [(false, &p.left), (true, &p.right)] {
                        r = r.child(if let Some(l) = l {
                            diff_line(l, true, right).grow(1.0)
                        } else {
                            row().min_w(498.0).h(21.0).grow(1.0).fill(color(RAISED))
                        });
                    }
                    r
                },
            ));
        bottom = bottom.child(sidebar).child(
            col()
                .grow(1.0)
                .min_w(0.0)
                .h_full()
                .min_h(0.0)
                .child(head)
                .child(code),
        );
    } else {
        bottom = bottom.child(
            label(
                if b.busy {
                    "Reading commit…"
                } else {
                    "Select a commit above to inspect its files and diff."
                },
                13.0,
                FAINT,
            )
            .padding(18.0),
        );
    }
    col()
        .w_full()
        .h_full()
        .fill(color(0xeceae4))
        .child(toolbar)
        .child(history)
        .child(
            row()
                .h(9.0)
                .center()
                .child(row().size(44.0, 3.0).radius(2.0).fill(color(0xd5d1c7))),
        )
        .child(bottom)
        .child(
            label(
                if !b.error.is_empty() {
                    b.error.clone()
                } else if b.busy {
                    "Reading repository…".into()
                } else {
                    "Read-only · All refs + HEAD · Merge changes against first parent".into()
                },
                11.0,
                MUTED,
            )
            .key("status")
            .padding(5.0),
        )
}
