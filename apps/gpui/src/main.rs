mod ui;
use gpui_kit::component::{
    Theme, ThemeMode,
    diff::{DiffFile, DiffState},
};
use gpui_kit::{prelude::*, *};
use rune_git_core::Browser;
use std::time::Duration;

struct GitApp {
    browser: Browser,
    diff: Entity<DiffState>,
    diff_error: Option<String>,
    split: bool,
}
impl GitApp {
    fn poll(&mut self, cx: &mut Context<Self>) {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(30))
                    .await;
                let busy = this.update(cx, |s, cx| {
                    if s.browser.poll() {
                        let files = DiffFile::parse(&s.browser.patch);
                        match files {
                            Ok(files) => {
                                s.diff.update(cx, |state, cx| state.set_files(files, cx));
                                s.diff_error = None;
                            }
                            Err(e) => s.diff_error = Some(format!("Could not display patch: {e}")),
                        }
                        cx.notify();
                    }
                    s.browser.busy
                });
                if !matches!(busy, Ok(true)) {
                    break;
                }
            }
        })
        .detach();
        cx.notify();
    }
    fn open(&mut self, cx: &mut Context<Self>) {
        if self.browser.busy {
            return;
        }
        let chosen = cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Open repository".into()),
        });
        cx.spawn(async move |this, cx| {
            let answer = chosen.await;
            let _ = this.update(cx, |s, cx| match answer {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.first() {
                        s.browser.open(path.clone());
                        s.poll(cx);
                    }
                }
                Ok(Ok(None)) => {}
                Ok(Err(e)) => {
                    s.browser.error = e.to_string();
                    cx.notify();
                }
                Err(e) => {
                    s.browser.error = e.to_string();
                    cx.notify();
                }
            });
        })
        .detach();
    }
}

fn main() {
    if std::env::var_os("GIT_CLIENT_DEBUG").is_some() {
        env_logger::init();
    }
    gpui_kit::application()
        .with_assets(gpui_kit::assets::Assets)
        .run(|cx: &mut App| {
            cx.text_system()
                .add_fonts(vec![
                    std::borrow::Cow::Borrowed(
                        include_bytes!("../../../assets/fonts/Inter-Regular.ttf").as_slice(),
                    ),
                    std::borrow::Cow::Borrowed(
                        include_bytes!("../../../assets/fonts/Inter-Medium.ttf").as_slice(),
                    ),
                    std::borrow::Cow::Borrowed(
                        include_bytes!("../../../assets/fonts/Inter-SemiBold.ttf").as_slice(),
                    ),
                    std::borrow::Cow::Borrowed(
                        include_bytes!("../../../assets/fonts/IBMPlexMono-Regular.ttf").as_slice(),
                    ),
                ])
                .expect("Could not register bundled fonts");
            gpui_kit::init(cx);
            let bounds = Bounds::centered(None, size(px(1440.), px(900.)), cx);
            gpui_kit::open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    window_min_size: Some(size(px(850.), px(550.))),
                    ..Default::default()
                },
                cx,
                |window, cx| {
                    Theme::change(ThemeMode::Light, Some(window), cx);
                    cx.global_mut::<Theme>().font_family = "Inter".into();
                    cx.global_mut::<Theme>().mono_font_family = "IBM Plex Mono".into();
                    cx.new(|cx| {
                        let mut s = GitApp {
                            browser: Browser::default(),
                            diff: cx.new(|cx| DiffState::new([], cx)),
                            diff_error: None,
                            split: false,
                        };
                        if let Some(path) = std::env::var_os("GIT_CLIENT_REPO") {
                            s.browser.open(path.into());
                            s.poll(cx);
                        }
                        s
                    })
                },
            )
            .expect("Could not open Git window");
            cx.activate(true);
            cx.on_window_closed(|cx, _| {
                if cx.windows().is_empty() {
                    cx.quit();
                }
            })
            .detach();
        });
}
