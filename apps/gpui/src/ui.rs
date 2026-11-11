use super::GitApp;
use gpui_kit::component::{
    Disableable, Selectable, Sizable, StyledExt,
    button::Button,
    diff::{Diff, DiffMode},
    resizable::{h_resizable, resizable_panel, v_resizable},
};
use gpui_kit::{prelude::*, *};
use rune_git_core::{
    GRAPH_COLORS,
    presentation::{avatar_color, file_parts, head_label, initials},
};
const INK: u32 = 0x191b20;
const MUTED: u32 = 0x525865;
const FAINT: u32 = 0x5f6572;
const LINE: u32 = 0xe4e1d9;
const RAISED: u32 = 0xf7f6f2;
const TEAL: u32 = 0x06748f;
const SELECT: u32 = 0xebf4f6;
fn avatar(name: &str) -> Div {
    div()
        .size(px(20.))
        .flex_shrink_0()
        .rounded_full()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgb(avatar_color(name)))
        .text_color(rgb(INK))
        .text_size(px(9.))
        .font_medium()
        .child(initials(name))
}
fn badge(label: impl Into<SharedString>, c: u32, head: bool) -> Div {
    div()
        .px_2()
        .py(px(2.))
        .rounded(px(6.))
        .border_1()
        .border_color(rgb(c).opacity(0.32))
        .bg(if head {
            rgb(c).into()
        } else {
            rgb(c).opacity(0.08)
        })
        .text_color(rgb(if head { 0xffffff } else { c }))
        .text_size(px(11.))
        .font_medium()
        .whitespace_nowrap()
        .child(label.into())
}
fn logo() -> impl IntoElement {
    div()
        .size(px(30.))
        .rounded(px(8.))
        .border_1()
        .border_color(rgb(0xd5d1c7))
        .bg(rgb(RAISED))
        .child(
            canvas(
                |_, _, _| (),
                |b, _, w, _| {
                    for (points, c) in [
                        (vec![[10., 7.], [10., 23.]], 0x9a5a00),
                        (vec![[10., 19.], [11., 15.], [18., 13.], [20., 9.]], TEAL),
                    ] {
                        let mut p = PathBuilder::stroke(px(2.2));
                        for (i, a) in points.iter().enumerate() {
                            let pt = point(b.left() + px(a[0]), b.top() + px(a[1]));
                            if i == 0 {
                                p.move_to(pt);
                            } else {
                                p.line_to(pt);
                            }
                        }
                        if let Ok(p) = p.build() {
                            w.paint_path(p, rgb(c));
                        }
                    }
                    for (x, y, c) in [(10., 22.5, 0x9a5a00), (20., 9., TEAL)] {
                        let mut p = PathBuilder::fill();
                        for i in 0..=20 {
                            let a = i as f32 * std::f32::consts::TAU / 20.;
                            let pt = point(
                                b.left() + px(x + 2.6 * a.cos()),
                                b.top() + px(y + 2.6 * a.sin()),
                            );
                            if i == 0 {
                                p.move_to(pt);
                            } else {
                                p.line_to(pt);
                            }
                        }
                        if let Ok(p) = p.build() {
                            w.paint_path(p, rgb(c));
                        }
                    }
                },
            )
            .size_full(),
        )
}
fn graph_cell(g: rune_git_core::GraphRow, width: f32) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, _, window, _| {
            let point_at = |x: f32, y: f32| point(bounds.left() + px(x), bounds.top() + px(y));
            for line in &g.lines {
                let mut path = PathBuilder::stroke(px(1.8));
                for (i, p) in line.points(30.0).iter().enumerate() {
                    if i == 0 {
                        path.move_to(point_at(p[0], p[1]));
                    } else {
                        path.line_to(point_at(p[0], p[1]));
                    }
                }
                if let Ok(path) = path.build() {
                    window.paint_path(path, rgb(rune_git_core::GRAPH_COLORS[line.color % 6]));
                }
            }
            let x = 14.0 + g.lane as f32 * 18.0;
            let mut node = PathBuilder::fill();
            for i in 0..=20 {
                let a = i as f32 * std::f32::consts::TAU / 20.0;
                let p = point_at(x + 4.0 * a.cos(), 15.0 + 4.0 * a.sin());
                if i == 0 {
                    node.move_to(p);
                } else {
                    node.line_to(p);
                }
            }
            if let Ok(path) = node.build() {
                window.paint_path(path, rgb(rune_git_core::GRAPH_COLORS[g.lane % 6]));
            }
        },
    )
    .w(px(width))
    .h(px(30.))
    .flex_shrink_0()
}
impl Render for GitApp {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let b = &self.browser;
        let gw = (b.graph_lanes as f32 * 18. + 28.).max(76.);
        let selected = b.selected.clone();
        let name = b
            .repo
            .as_ref()
            .and_then(|r| r.path.file_name())
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Arbor".into());
        let path = b
            .repo
            .as_ref()
            .map(|r| r.path.display().to_string())
            .unwrap_or_else(|| "Open a local repository".into());
        let history = div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(rgb(0xffffff))
            .child(
                div()
                    .h(px(40.))
                    .px_4()
                    .flex()
                    .items_center()
                    .gap_4()
                    .border_b_1()
                    .border_color(rgb(LINE))
                    .child(div().font_semibold().child("History"))
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(FAINT))
                            .child(format!("All branches · {} commits", b.commits.len())),
                    ),
            )
            .child(
                div()
                    .h(px(28.))
                    .px_2()
                    .flex()
                    .items_center()
                    .border_b_1()
                    .border_color(rgb(LINE))
                    .text_size(px(10.))
                    .text_color(rgb(FAINT))
                    .child(div().w(px(gw)).child("GRAPH"))
                    .child(div().flex_1().child("DESCRIPTION"))
                    .child(div().w(px(156.)).child("AUTHOR"))
                    .child(div().w(px(104.)).child("DATE"))
                    .child(div().w(px(80.)).child("COMMIT")),
            )
            .child(
                uniform_list(
                    "commits",
                    b.commits.len(),
                    cx.processor(move |s, range: std::ops::Range<usize>, _, cx| {
                        range
                            .map(|i| {
                                let c = &s.browser.commits[i];
                                let oid = c.oid.clone();
                                let on = selected.as_deref() == Some(&c.oid);
                                let mut description =
                                    div().flex().items_center().gap(px(6.)).flex_1().min_w_0();
                                for r in c.refs.split(", ").filter(|r| !r.is_empty()).take(3) {
                                    let head = r.starts_with("HEAD -> ");
                                    description = description.child(
                                        badge(
                                            r.strip_prefix("HEAD -> ").unwrap_or(r).to_owned(),
                                            GRAPH_COLORS[s.browser.graph[i].lane % 6],
                                            head,
                                        )
                                        .max_w(px(220.))
                                        .truncate(),
                                    );
                                }
                                div()
                                    .id(i)
                                    .w_full()
                                    .h(px(30.))
                                    .px_2()
                                    .flex()
                                    .items_center()
                                    .cursor_pointer()
                                    .bg(rgb(if on { SELECT } else { 0xffffff }))
                                    .hover(|s| s.bg(rgb(0xf2f0eb)))
                                    .child(graph_cell(s.browser.graph[i].clone(), gw))
                                    .child(
                                        description.child(
                                            div()
                                                .flex_1()
                                                .min_w_0()
                                                .truncate()
                                                .text_color(rgb(if c.parents.len() > 1 {
                                                    MUTED
                                                } else {
                                                    INK
                                                }))
                                                .child(c.subject.clone()),
                                        ),
                                    )
                                    .child(
                                        div()
                                            .w(px(156.))
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .child(avatar(&c.author))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_w_0()
                                                    .truncate()
                                                    .text_xs()
                                                    .text_color(rgb(MUTED))
                                                    .child(c.author.clone()),
                                            ),
                                    )
                                    .child(
                                        div()
                                            .w(px(104.))
                                            .text_xs()
                                            .text_color(rgb(FAINT))
                                            .child(c.date[..10].to_owned()),
                                    )
                                    .child(
                                        div()
                                            .w(px(80.))
                                            .font_family("IBM Plex Mono")
                                            .text_xs()
                                            .text_color(rgb(FAINT))
                                            .child(c.oid[..8].to_owned()),
                                    )
                                    .on_click(cx.listener(move |s, _, _, cx| {
                                        if !s.browser.busy {
                                            s.browser.select(oid.clone());
                                            s.poll(cx);
                                        }
                                    }))
                            })
                            .collect()
                    }),
                )
                .flex_1()
                .min_h_0(),
            )
            .when(b.more, |p| {
                p.child(
                    div().flex().px_2().py_1().child(
                        Button::new("more")
                            .label("Load more commits")
                            .small()
                            .disabled(b.busy)
                            .on_click(cx.listener(|s, _, _, cx| {
                                s.browser.load_more();
                                s.poll(cx);
                            })),
                    ),
                )
            })
            .when(b.commits.is_empty() && !b.busy, |p| {
                p.child(
                    div()
                        .p_5()
                        .text_color(rgb(FAINT))
                        .child("Open a repository to explore its branches and commits."),
                )
            });
        let mut bottom = div().size_full().min_h_0().flex().bg(rgb(0xffffff));
        if let Some(d) = &b.detail {
            let (title, body) = d.message.split_once('\n').unwrap_or((&d.message, ""));
            let mut parents = div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(6.))
                .child(badge(d.oid[..8].to_owned(), MUTED, false));
            for p in d.parents.split_whitespace() {
                let oid = p.to_owned();
                parents = parents
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(rgb(FAINT))
                            .child("Parent"),
                    )
                    .child(
                        Button::new(SharedString::from(format!("parent-{p}")))
                            .label(p[..8].to_owned())
                            .small()
                            .on_click(cx.listener(move |s, _, _, cx| {
                                if !s.browser.busy {
                                    s.browser.select(oid.clone());
                                    s.poll(cx);
                                }
                            })),
                    );
            }
            let summary = div()
                .id("commit-message")
                .max_h(px(240.))
                .overflow_y_scroll()
                .p_4()
                .flex()
                .flex_col()
                .gap(px(10.))
                .border_b_1()
                .border_color(rgb(LINE))
                .child(
                    div()
                        .font_semibold()
                        .text_size(px(15.))
                        .child(title.to_owned()),
                )
                .when(!body.trim().is_empty(), |p| {
                    p.child(
                        div()
                            .text_xs()
                            .text_color(rgb(MUTED))
                            .child(body.trim().to_owned()),
                    )
                })
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap_2()
                        .child(avatar(&d.author))
                        .child(div().text_xs().child(d.author.clone()))
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(rgb(FAINT))
                                .child(d.date[..10].to_owned()),
                        ),
                )
                .child(parents);
            let adds = d.files.iter().map(|f| f.additions).sum::<usize>();
            let dels = d.files.iter().map(|f| f.deletions).sum::<usize>();
            let sidebar = div()
                .size_full()
                .min_h_0()
                .flex()
                .flex_col()
                .child(summary)
                .child(
                    div()
                        .h(px(32.))
                        .px_4()
                        .flex()
                        .items_center()
                        .gap_2()
                        .text_size(px(10.))
                        .text_color(rgb(FAINT))
                        .child(
                            div()
                                .flex_1()
                                .child(format!("{} FILES CHANGED", d.files.len())),
                        )
                        .child(div().text_color(rgb(0x1b6e2d)).child(format!("+{adds}")))
                        .child(div().text_color(rgb(0xae2016)).child(format!("−{dels}"))),
                )
                .child(
                    uniform_list(
                        "files",
                        d.files.len(),
                        cx.processor(|s, range: std::ops::Range<usize>, _, cx| {
                            range
                                .map(|i| {
                                    let f = &s.browser.detail.as_ref().unwrap().files[i];
                                    let (dir, name) = file_parts(&f.path);
                                    let on = s.browser.selected_file == Some(i);
                                    let c = match f.status.as_bytes()[0] {
                                        b'A' => 0x1b6e2d,
                                        b'D' => 0xae2016,
                                        _ => 0x9a5a00,
                                    };
                                    div().id(i).h(px(44.)).w_full().px(px(6.)).child(
                                        div()
                                            .id(("file-hit", i))
                                            .h_full()
                                            .flex()
                                            .items_center()
                                            .gap_2()
                                            .px_2()
                                            .rounded(px(8.))
                                            .border_1()
                                            .border_color(if on {
                                                rgb(TEAL).opacity(0.35)
                                            } else {
                                                rgb(0xffffff).opacity(0.0)
                                            })
                                            .bg(rgb(if on { SELECT } else { 0xffffff }))
                                            .cursor_pointer()
                                            .child(badge(f.status[..1].to_owned(), c, false))
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .min_w_0()
                                                    .flex()
                                                    .flex_col()
                                                    .gap(px(2.))
                                                    .child(
                                                        div()
                                                            .text_xs()
                                                            .font_medium()
                                                            .truncate()
                                                            .child(name.to_owned()),
                                                    )
                                                    .child(
                                                        div()
                                                            .text_size(px(11.))
                                                            .text_color(rgb(FAINT))
                                                            .truncate()
                                                            .child(if dir.is_empty() {
                                                                "Repository root".to_owned()
                                                            } else {
                                                                dir.to_owned()
                                                            }),
                                                    ),
                                            )
                                            .child(
                                                div()
                                                    .text_size(px(11.))
                                                    .text_color(rgb(MUTED))
                                                    .child(if f.binary {
                                                        "Binary".into()
                                                    } else {
                                                        format!("+{} −{}", f.additions, f.deletions)
                                                    }),
                                            )
                                            .on_click(cx.listener(move |s, _, _, cx| {
                                                if !s.browser.busy {
                                                    s.browser.select_file(i);
                                                    s.poll(cx);
                                                }
                                            })),
                                    )
                                })
                                .collect()
                        }),
                    )
                    .flex_1()
                    .min_h_0(),
                );
            let file = b.selected_file.and_then(|i| d.files.get(i));
            let mut diff = div().size_full().min_h_0().min_w_0().flex().flex_col();
            let mut header = div()
                .min_h(px(52.))
                .px_4()
                .py_2()
                .flex()
                .items_center()
                .gap_3()
                .border_b_1()
                .border_color(rgb(LINE));
            if let Some(f) = file {
                let (dir, name) = file_parts(&f.path);
                let c = match f.status.as_bytes()[0] {
                    b'A' => 0x1b6e2d,
                    b'D' => 0xae2016,
                    _ => 0x9a5a00,
                };
                header = header
                    .child(badge(f.status[..1].to_owned(), c, false))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .items_center()
                            .font_family("IBM Plex Mono")
                            .child(
                                div()
                                    .truncate()
                                    .text_xs()
                                    .text_color(rgb(FAINT))
                                    .child(dir.to_owned()),
                            )
                            .child(div().text_sm().font_semibold().child(name.to_owned())),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0x1b6e2d))
                            .child(format!("+{}", f.additions)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(rgb(0xae2016))
                            .child(format!("−{}", f.deletions)),
                    );
            }
            header = header.child(
                div()
                    .flex()
                    .p(px(2.))
                    .rounded(px(8.))
                    .bg(rgb(0xf0eee8))
                    .border_1()
                    .border_color(rgb(LINE))
                    .child(
                        Button::new("unified")
                            .label("Unified")
                            .small()
                            .selected(!self.split)
                            .on_click(cx.listener(|s, _, _, cx| {
                                s.split = false;
                                s.diff
                                    .update(cx, |state, cx| state.set_mode(DiffMode::Unified, cx));
                                cx.notify();
                            })),
                    )
                    .child(
                        Button::new("split")
                            .label("Split")
                            .small()
                            .selected(self.split)
                            .on_click(cx.listener(|s, _, _, cx| {
                                s.split = true;
                                s.diff
                                    .update(cx, |state, cx| state.set_mode(DiffMode::Split, cx));
                                cx.notify();
                            })),
                    ),
            );
            diff = diff.child(header);
            if b.busy {
                diff = diff.child(div().p_5().text_color(rgb(FAINT)).child("Reading file…"));
            } else if d.files.is_empty() {
                diff = diff.child(
                    div()
                        .p_5()
                        .child("This commit has no file changes against its first parent."),
                );
            } else if let Some(e) = &self.diff_error {
                diff = diff.child(div().p_4().child(e.clone()));
            } else {
                diff = diff.child(
                    Diff::new(&self.diff)
                        .size_full()
                        .header_visible(false)
                        .line_number(true)
                        .soft_wrap(false),
                );
            }
            bottom = bottom.child(
                h_resizable("file-diff-split")
                    .child(
                        resizable_panel()
                            .size(px(360.))
                            .size_range(px(260.)..px(550.))
                            .child(sidebar),
                    )
                    .child(resizable_panel().child(diff)),
            );
        } else {
            bottom = bottom.child(
                div()
                    .p_5()
                    .text_color(rgb(FAINT))
                    .child("Select a commit to inspect its files and diff."),
            );
        }
        let status = if !b.error.is_empty() {
            b.error.clone()
        } else if b.busy {
            "Reading repository…".into()
        } else {
            "Read-only · All refs + HEAD · Merge changes against first parent".into()
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(rgb(0xeceae4))
            .text_color(rgb(INK))
            .text_size(px(13.))
            .child(
                div()
                    .h(px(60.))
                    .px_4()
                    .flex()
                    .items_center()
                    .gap_3()
                    .bg(rgb(0xffffff))
                    .border_b_1()
                    .border_color(rgb(LINE))
                    .child(logo())
                    .child(
                        div()
                            .w(px(260.))
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(
                                div()
                                    .font_semibold()
                                    .text_size(px(15.))
                                    .truncate()
                                    .child(name),
                            )
                            .child(
                                div()
                                    .text_size(px(11.))
                                    .text_color(rgb(FAINT))
                                    .truncate()
                                    .child(path),
                            ),
                    )
                    .child(badge(head_label(&b.commits), TEAL, false))
                    .child(div().flex_1())
                    .child(
                        Button::new("open")
                            .label("Open repository")
                            .small()
                            .disabled(b.busy)
                            .on_click(cx.listener(|s, _, _, cx| s.open(cx))),
                    )
                    .child(
                        Button::new("refresh")
                            .label("Refresh")
                            .small()
                            .disabled(b.busy || b.repo.is_none())
                            .on_click(cx.listener(|s, _, _, cx| {
                                s.browser.refresh();
                                s.poll(cx);
                            })),
                    ),
            )
            .child(
                div().flex_1().min_h_0().child(
                    v_resizable("history-changes-split")
                        .child(
                            resizable_panel()
                                .size(px(392.))
                                .size_range(px(180.)..px(1000.))
                                .child(history),
                        )
                        .child(
                            resizable_panel()
                                .size_range(px(220.)..px(1200.))
                                .child(bottom),
                        ),
                ),
            )
            .child(
                div()
                    .h(px(24.))
                    .px_3()
                    .flex()
                    .items_center()
                    .bg(rgb(0xeceae4))
                    .text_size(px(11.))
                    .text_color(rgb(MUTED))
                    .child(status),
            )
    }
}
