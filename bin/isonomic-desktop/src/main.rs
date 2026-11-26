// Copyright 2026 Anud Inc.
// SPDX-License-Identifier: GPL-3.0-or-later

//! Isonomic Desktop: the window. Finds or starts the engine, opens the
//! shell, and owns the menu bar icon. Closing the window hides it; Quit
//! stops the engine in order, then the window.

extern crate ui as gpui;

mod engine_notices;
mod problems;
mod self_check;
mod setup;
mod window_size;

use std::sync::Arc;

use engine_link::agent::LoginAgent;
use engine_link::settings::FileSettings;
use engine_link::{EngineLink, LinkConfig};
use futures::StreamExt as _;
use isonomic_app::{Registration, SettingsStore};
use setup::FirstRunSetup;
use shell::apps::AppHost;
use shell::menubar::LoginToggle;
use shell::onboarding::backend::Backend;
use shell::onboarding::engine::EngineBackend;
use shell::{lifecycle, menubar, Shell};
use ui::{AppContext as _, KeyBinding, Menu, MenuItem};

ui::actions!(isonomic, [Quit, Hide, Acknowledgements]);

/// App settings in the Capsule, through the engine link.
struct CapsuleSettings(FileSettings);

impl SettingsStore for CapsuleSettings {
    fn get(&self, app: &str, key: &str) -> Option<serde_json::Value> {
        self.0.get(app, key)
    }

    fn set(&self, app: &str, key: &str, value: serde_json::Value) {
        self.0.set(app, key, value)
    }

    fn remove(&self, app: &str, key: &str) {
        self.0.remove(app, key)
    }
}

/// The app crates compiled into the window, so a crash can be blamed on
/// the app whose code it was in.
fn app_crates() -> Vec<reports::panic::AppCrate> {
    use isonomic_app::App as _;
    let chat = isonomic_chat::Chat::manifest();
    #[allow(unused_mut)]
    let mut crates = vec![reports::panic::AppCrate {
        prefix: reports::panic::crate_of::<isonomic_chat::Chat>(),
        id: chat.id,
        version: chat.version,
    }];
    #[cfg(feature = "example-app")]
    {
        let hello = hello_app::Hello::manifest();
        crates.push(reports::panic::AppCrate {
            prefix: reports::panic::crate_of::<hello_app::Hello>(),
            id: hello.id,
            version: hello.version,
        });
    }
    crates
}

/// The first-party app crates compiled into the window, in rail order.
fn first_party() -> Vec<Registration> {
    vec![
        isonomic_app::register::<isonomic_chat::Chat>(),
        #[cfg(feature = "example-app")]
        isonomic_app::register::<hello_app::Hello>(),
    ]
}

/// The menu's switch, backed by the login agent.
struct AtLogin(Arc<LoginAgent>);

impl LoginToggle for AtLogin {
    fn enabled(&self) -> bool {
        self.0.enabled()
    }

    fn set_enabled(&self, on: bool) -> Result<(), String> {
        self.0
            .set_enabled(on)
            .map(|_| ())
            .map_err(|error| error.to_string())
    }
}

fn main() {
    let config = LinkConfig::from_environment().unwrap_or_else(|error| {
        eprintln!("[isonomic-desktop] fatal: {error}");
        std::process::exit(1);
    });
    // Before the link and the event loop: a missing Capsule is created, so
    // the engine starts in it and first run knows it is new.
    let capsule = config.capsule();
    let state = capsule.prepare().unwrap_or_else(|error| {
        eprintln!("[isonomic-desktop] fatal: cannot create the Capsule: {error}");
        std::process::exit(1);
    });
    // Before the link and before the event loop: a missing engine is then
    // started through the agent, and nothing here blocks a frame.
    // A self-check runs in a scratch Capsule: it must never install or
    // change the person's own login agent, so it runs without one.
    let check = self_check::mode();
    // Problem reports: the window log, the panic hook, the hang watchdog
    // and the crash-loop guard, before anything else can fail.
    let problems = problems::start(&config.root, app_crates(), check.is_some());
    let (report_context, beat, safe) = (
        Arc::clone(&problems.context),
        problems.beat.clone(),
        problems.safe,
    );
    let guard = Arc::new(problems.guard);
    // Opened from the disk image or Downloads without being moved: macOS
    // runs it from a path that changes every launch, so no login agent may
    // point at it, and the person is asked to move it.
    let translocated = std::env::current_exe()
        .map(|exe| engine_link::translocated(&exe))
        .unwrap_or(false);
    let agent = if check.is_some() || translocated {
        None
    } else {
        config.login_agent().map(Arc::new)
    };
    // `isonomic://` links given as launch arguments, so a link can be opened
    // without going through the system's URL handling.
    let launch_links: Vec<String> = std::env::args()
        .skip(1)
        .filter(|arg| arg.starts_with("isonomic://"))
        .collect();
    if let Some(agent) = &agent {
        match agent.sync() {
            Ok(change) => reports::log::line(&format!("start at login: {change:?}")),
            Err(error) => reports::log::line(&format!("start at login not set up: {error}")),
        }
    }
    // Read before the event loop; later writes run on their own thread.
    let settings = FileSettings::open(&config.root).unwrap_or_else(|error| {
        eprintln!("[isonomic-desktop] fatal: cannot read app settings: {error}");
        std::process::exit(1);
    });
    let settings: Arc<dyn SettingsStore> = Arc::new(CapsuleSettings(settings));
    // A self-check clicks through the window; no notice may cover it.
    let notices_root = config.root.clone();
    let size_root = config.root.clone();
    let remembered = window_size::load(&config.root);
    let kept = check
        .is_none()
        .then(|| Arc::new(reports::kept::Kept::in_capsule(&config.root)));
    let link = EngineLink::start(config, agent.clone()).unwrap_or_else(|error| {
        eprintln!("[isonomic-desktop] fatal: {error}");
        std::process::exit(1);
    });
    // The link owns the connection: it lives for the whole process.
    let link: &'static EngineLink = Box::leak(Box::new(link));

    // Isonomic UI's icons are embedded in its asset source, which the app
    // registers: without it every icon draws nothing.
    let app = ui::application().with_assets(ui::assets::AllAssets);
    // Dock icon clicked while the window is hidden.
    app.on_reopen(lifecycle::show);
    // Deep links (isonomic://...) arrive outside the app's context; they
    // are queued here and handed to the shell once its window is open.
    let (links, mut opened_links) = futures::channel::mpsc::unbounded::<String>();
    if check.is_none() {
        for url in &launch_links {
            let _ = links.unbounded_send(url.clone());
        }
    }
    app.on_open_urls(move |urls| {
        for url in urls {
            let _ = links.unbounded_send(url);
        }
    });
    app.run(move |cx| {
        ui::init(cx);
        let client = link.client().clone();

        cx.bind_keys([
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("cmd-w", Hide, None),
        ]);
        let quit_client = client.clone();
        let quit_guard = Arc::clone(&guard);
        cx.on_action(move |_: &Quit, cx| {
            // A clean quit is a good launch.
            quit_guard.survived();
            lifecycle::quit(quit_client.clone(), cx)
        });
        // The watchdog's beat comes from this thread, so a stuck frame shows.
        cx.spawn(async move |cx| loop {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(1))
                .await;
            beat.beat();
        })
        .detach();
        // A launch that lasts this long resets the crash-loop guard.
        let survived = Arc::clone(&guard);
        cx.spawn(async move |cx| {
            cx.background_executor()
                .timer(std::time::Duration::from_secs(
                    reports::guard::SURVIVED_SECS,
                ))
                .await;
            survived.survived();
        })
        .detach();
        cx.on_action(|_: &Hide, cx| cx.hide());
        cx.set_menus([Menu::new("Isonomic").items([
            MenuItem::action("Acknowledgements", Acknowledgements),
            MenuItem::separator(),
            MenuItem::action("Hide Isonomic", Hide),
            MenuItem::separator(),
            MenuItem::action("Quit Isonomic", Quit),
        ])]);

        // A new person meets the onboarding window first; the app's window
        // opens when they start. A self-check drives the app's window by
        // its ids, so it goes straight there.
        let onboard = check.is_none() && !state.set_up;
        // Onboarding works over the engine, the Capsule's settings and
        // the login agent.
        let onboarding = onboard.then(|| {
            let login = agent
                .clone()
                .map(|agent| Arc::new(AtLogin(agent)) as Arc<dyn LoginToggle>);
            let settings = Arc::clone(&settings);
            Arc::new(EngineBackend::new(client.clone(), settings, login)) as Arc<dyn Backend>
        });
        let open_app = move |cx: &mut ui::App| {
            let options = lifecycle::window_options(cx, remembered);
            let live = client.live();
            let logs = client.clone();
            // Settings live in the Capsule until the engine stores them.
            // Apps' window requests (installing a model) queue here until the
            // shell exists to serve them.
            let (requests, queued) = shell::install::requests();
            let host = AppHost::new(client.clone(), settings).with_requests(requests);
            let request_client = client.clone();
            let host_client = client.clone();
            let profile_client = client.clone();
            let (window, shell) = ui::open_window(options, cx, |window, cx| {
                lifecycle::hide_on_close(window, cx);
                cx.new(|cx| {
                    // The size the person leaves the window at is the next one.
                    let size_root = size_root.clone();
                    cx.observe_window_bounds(window, move |_, window, cx| {
                        let (root, now) = (size_root.clone(), window.bounds().size);
                        cx.background_spawn(async move { window_size::save(&root, now) })
                            .detach();
                    })
                    .detach();
                    let mut shell = Shell::new(live, Arc::new(logs), cx);
                    shell.serve_requests(queued, Arc::new(request_client), cx);
                    let problems = link.problems();
                    shell
                        .home()
                        .update(cx, |home, cx| home.follow_problems(problems, cx));
                    if translocated {
                        shell.home().update(cx, |home, cx| {
                            home.add_notice(
                                "Move Isonomic to Applications",
                                "Move Isonomic to the Applications folder and open it from there. \
                             Until then it can't start at login."
                                    .into(),
                                cx,
                            )
                        });
                    }
                    if safe {
                        // The crash-loop guard: no app this time, and Home says why.
                        shell.home().update(cx, |home, cx| {
                            home.add_notice(problems::SAFE_TITLE, problems::SAFE_NOTICE.into(), cx)
                        });
                    } else {
                        for app in first_party() {
                            shell.mount(app, host.clone(), cx);
                        }
                    }
                    if let Some(kept) = kept.clone() {
                        shell.watch_reports(kept, cx);
                        if let Some(reports) = shell.reports() {
                            let writer = Arc::clone(&report_context);
                            reports.update(cx, |r, _| r.write_with(writer));
                            // Send is offered only when the build has a destination.
                            if let Some(sender) = report_sender() {
                                let host = sender.destination().host().to_string();
                                reports.update(cx, |r, cx| r.send_with(Arc::new(sender), host, cx));
                            }
                        }
                        shell.review_on_status(profile_client.live(), cx);
                        let id_client = profile_client.clone();
                        shell.report_identity(
                            Box::pin(async move {
                                let mut live = id_client.live();
                                let _ = live.wait_for(|live| live.connected).await;
                                shell::first_run::identity_from(&id_client).await
                            }),
                            cx,
                        );
                    }
                    let tab_context = Arc::clone(&report_context);
                    shell.on_tab_change(move |id| {
                        if let Ok(mut tab) = tab_context.tab.lock() {
                            *tab = Some(id.to_string());
                        }
                    });
                    // The hardware profile caps how many apps stay loaded.
                    let profile_client = profile_client.clone();
                    let profile_context = Arc::clone(&report_context);
                    shell.follow_profile(
                        Box::pin(async move {
                            // Ask once the engine is connected.
                            let mut live = profile_client.live();
                            let _ = live.wait_for(|live| live.connected).await;
                            let profile = profile_client.profile().await;
                            if let Ok(p) = &profile {
                                problems::profile_known(
                                    &profile_context,
                                    &p.profile,
                                    p.total_memory_bytes,
                                );
                            }
                            profile.map(|p| p.profile).map_err(|e| e.to_string())
                        }),
                        cx,
                    );
                    if onboard {
                        // Onboarding was the setup: no first run after it.
                        cx.background_spawn(async move {
                            let _ = capsule.mark_set_up();
                        })
                        .detach();
                    } else if !state.set_up {
                        let setup = FirstRunSetup {
                            client: host_client,
                            capsule,
                        };
                        shell.start_first_run(Arc::new(setup), state.created, cx);
                    }
                    shell
                })
            })
            .expect("failed to open the window");
            // Keyboard focus starts on the rail (or first run).
            let _ = window.update(cx, |_, window, cx| {
                shell.update(cx, |shell, cx| shell.focus_rail(window, cx))
            });
            // After onboarding comes Chat, once its engine is connected.
            if onboard {
                let (chat, mut live) = (shell.clone(), client.live());
                cx.spawn(async move |cx| {
                    let _ = live.wait_for(|live| live.connected).await;
                    let _ = cx.update_window(window, |_, window, cx| {
                        chat.update(cx, |shell, cx| {
                            shell.open_link("isonomic://chat/home", window, cx);
                        })
                    });
                })
                .detach();
            }
            cx.on_action(move |_: &Acknowledgements, cx| {
                lifecycle::show(cx);
                cx.spawn(async move |cx| {
                    let notices = cx.background_spawn(async { notices() }).await;
                    let _ = cx.update_window(window, |_, window, cx| {
                        shell::acknowledgements::open(notices, window, cx)
                    });
                })
                .detach();
            });
            // What the engine said at start that the person should see.
            let notice_shell = shell.clone();
            let notice_client = client.clone();
            cx.spawn(async move |cx| {
                let mut live = notice_client.live();
                let _ = live.wait_for(|live| live.connected).await;
                let found = cx
                    .background_spawn(async move { engine_notices::unseen(&notices_root) })
                    .await;
                if let Some(message) = found {
                    notice_shell.update(cx, |shell, cx| {
                        shell.home().update(cx, |home, cx| {
                            home.add_notice(engine_notices::SET_ASIDE_TITLE, message, cx)
                        })
                    });
                }
            })
            .detach();
            let link_shell = shell.clone();
            cx.spawn(async move |cx| {
                while let Some(url) = opened_links.next().await {
                    let _ = cx.update_window(window, |_, window, cx| {
                        lifecycle::show(cx);
                        let handled =
                            link_shell.update(cx, |shell, cx| shell.open_link(&url, window, cx));
                        if !handled {
                            reports::log::line(&format!("no app opens {url}"));
                        }
                    });
                }
            })
            .detach();

            if let Some(mode) = check {
                self_check::start(mode, window, shell, client, launch_links, cx);
                return;
            }
            let login = agent.map(|agent| Arc::new(AtLogin(agent)) as Arc<dyn LoginToggle>);
            menubar::install(client, link.client().live(), login, cx);
        };
        match onboarding {
            Some(backend) => drop(shell::onboarding::open(cx, backend, open_app)),
            None => open_app(cx),
        }
    });
}

/// The licence notices the app carries: `Contents/Resources/NOTICES.txt`,
/// beside `Contents/MacOS/` where the window runs. None outside a bundle.
fn notices() -> Option<String> {
    let exe = std::env::current_exe().ok()?;
    let contents = exe.parent()?.parent()?;
    std::fs::read_to_string(contents.join("Resources").join("NOTICES.txt")).ok()
}

/// The problem-report sender, when this build has a destination.
fn report_sender() -> Option<reports::send::HttpSender> {
    let to = reports::send::Destination::configured()?;
    match reports::send::HttpSender::new(to) {
        Ok(sender) => Some(sender),
        Err(error) => {
            reports::log::line(&format!("no report sender: {}", error.kind()));
            None
        }
    }
}
