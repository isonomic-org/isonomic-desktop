# Settings assets

What the Settings pages hold and show. `crates/shell/src/settings` reads these files; `docs/architecture/settings/` says how.

- `<name>.json`: one file for each page, tab, sub-page and dialog of the design (Isonomic Desktop, "Settings 0.3"), made from its frame: the frame's layers in order, with the design's own words and sample contents. A page is a list of blocks (`k`): `header`, `section`, `group` (its `rows`), `tabs`, `back`, `note`, `empty`, `filter`, `table`, `chart`, `storage`, `cards`, `textarea`; a dialog adds `title`, `field`, `actions`, `plans`, `words`, `code`, `keys`, `log` and `dropzone`. A row or a heading ends in controls (`c`): `toggle`, `select`, `seg`, `button`, `iconbtn`, `input`, `textarea`, `slider`, `kbd`, `badge`, `status`, `text`, `icon`. Icons are named as the library's files are.
- `nav.json`: the sidebar's sections and their pages, in the design's order.
- `links.json`: what a button or a row opens, as `page|row|label`: `d:` and a dialog's name, or `p:` and a page's. A row that leads to a page of its own has the label `>`.
- `logos/*.png`: the marks only Settings shows beside a model, an app or a file, the design's own pictures (its `Logo/…` components), each in its 24-point box, exported at four pixels to a point, unchanged. The marks remain their owners' trademarks. The other marks are the window's shared ones, under `assets/onboarding/logos`.

A change to a page is a change to its file; the files are no longer made from the design by a tool in this repository.
