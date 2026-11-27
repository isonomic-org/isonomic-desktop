# App icon

The Isonomic app icon, from the design file "Isonomic App Icons" (frame `BU-black-bg`), supplied by Omar on 15 Nov 2026.

- `app-icon.svg`: the source, 140 by 140 units, a dark rounded square with the mark, transparent outside the rounded square.
- `app-icon-1024.png`: the same artwork rendered at 1024 by 1024 pixels with an alpha channel.
- `menu-bar-mark.svg`: the mark alone from the same artwork, in black on clear, for the menu bar.
- `menu-bar-mark.png`: that mark at 36 by 36 pixels, shown at 18 points as a template image.

The app icon's artwork fills its canvas edge to edge. A macOS app icon sits inside a margin on a 1024 pixel canvas, so the bundle step should place this artwork on the macOS icon grid when it builds the `.icns`, not use the PNG as is.
