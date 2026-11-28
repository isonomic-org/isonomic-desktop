# Settings: spec

The design is Isonomic Desktop, "Settings 0.3": 35 pages in four sections (Personal, Engine, Integrations, Coding), their tabs and sub-pages, and the dialogs they open. This area builds every one of them in the window.

## Where it is

- **In the window.** A gear at the foot of the rail, and ⌘, open Settings in the place of the tabs; choosing a tab puts it away. The window offers Settings only where it is told to (`Shell::offer_settings`); the app does so when `ISONOMIC_SETTINGS` is set.
- **The view** is `shell::settings::Settings`: the list of pages down the left with its search, and the page on show in a column the design's width.
- **The pages are files**, `assets/settings/<name>.json`, one for each page, tab, sub-page and dialog, read once (`settings::spec`). `assets/settings/README.md` says what a file holds. A page is changed in its file.

## What it is drawn with

Every control is Isonomic UI's own component: the sidebar and its groups, the group box for a panel, the switch, the button and its dropdown menu for a choice, the toggle group for a segment, the tab bar, the input and the text area, the slider, the tag, the keys, the avatar, the table, the bar chart, the segmented bar, the empty state, the dialog and the notification. Icons are the library's set, by name. Sizes are `ui::settings`; colours are the theme's.

## What it keeps

A switch, a choice, a segment and the companion chosen are kept in the Capsule's settings, under `isonomic` / `settings`, as one object keyed by `page|row|control`. They are read when Settings opens and written when one changes, both off the foreground thread.

## What moves

- The sidebar goes to a page; a page's tabs go to the tab's own page; a row with an arrow goes to its sub-page, whose "back" returns.
- A button opens the dialog the design gives it (`assets/settings/links.json`).
- The sidebar's search lists the pages and settings whose titles match; a page's own search narrows its rows.

## What is not here yet

Nothing on these pages reaches the engine: the contents are the design's samples, and a button with nothing behind it says "This is not connected yet." `status.md` lists what each page waits for.
