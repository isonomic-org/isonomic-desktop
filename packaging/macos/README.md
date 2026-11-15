# macOS packaging inputs

- `AppIcon-1024.png`: the app icon, 1024 by 1024 pixels, PNG with transparency. `cargo xtask bundle` turns it into `Contents/Resources/AppIcon.icns` and names it in `Info.plist`. Leave it out and the app has no icon; the bundle says so and carries on. No icon is approved yet.
