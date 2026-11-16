# The window: isonomic-desktop

Spec section 6.

```
crates/
  ui           the single re-export of the component library
  shell        window, rail, tab host, menu bar icon, deep links, single instance
  home  catalog  activity  settings
  engine-link  finds, starts and reconnects to the engine
bin/isonomic-desktop
packaging/macos
```

- **`ui` is one line.** It re-exports Isonomic UI, a sibling checkout pinned to the commit in `UI_REF`, the way the SDK is pinned by `SDK_REF`. Nothing else in the repo names Isonomic UI or GPUI.
- **State.** Each tab is a GPUI entity. Shell state (rail order, open tabs, window frames) is stored in the engine, not in a local file, so it survives reinstall and follows the user.
- **Engine link.** One connection per process, multiplexed. Views subscribe to events; they never poll.
- **Threading.** GPUI's foreground thread draws. Every engine call is async on the background executor. No blocking call on the foreground thread, ever.

## Things GPUI does not give us

| Need | Plan |
| --- | --- |
| Menu bar icon | Native status item on macOS, owned by the window process. First spike |
| Start at login | A per-user launch agent for the engine, installed on first run, removable in Settings |
| Single instance | The window holds a lock next to the socket; a second launch forwards its arguments and exits |
| Deep links | Registered in the app bundle; GPUI delivers opened URLs to the shell |
| Passkeys | Native platform calls. Spike before the vault design is frozen |
| Web content (card checkout, sign-in pages) | The system browser with a return link. No embedded web view in the shell |
| Updates | Deferred with signing |

## Budgets

- First frame within 400 ms with a warm engine.
- Tab switch within one frame for a loaded tab.
- Idle window under 150 MB with no app tabs open.
- No raw colors, sizes or durations outside the `ui` crate's tokens.

## Milestone 0 scope

`ui`, `engine-link`, `shell` with the rail, a tab host and one Home tab showing each service's live state from a subscription, the menu bar icon (status light, Open, Quit), and a universal unsigned app. The first window build must be confirmed on the 2020 Intel test machine before any UI investment.
