# Opening Isonomic the first time

Isonomic is not yet signed with an Apple developer identity, so macOS stops it the first time you open it. That happens once; after that it opens like any other app.

1. Drag **Isonomic** into **Applications**, then open it from there. Opening it straight from the disk image or the Downloads folder makes it forget its settings between launches.
2. Double-click Isonomic. macOS says it can't check the app for malicious software and offers **Done** or **Move to Trash**. Choose **Done**.
3. Open **System Settings**, then **Privacy & Security**. Scroll down to the message about Isonomic and click **Open Anyway**. Enter your password if asked.
4. Open Isonomic again and click **Open** in the dialog that follows.

To check the download before opening it, compare its SHA-256 with the `.sha256` file beside it:

```
shasum -a 256 Isonomic-*.dmg
```

Isonomic runs on a Mac with macOS 13.3 or later and at least 16 GB of memory, Intel or Apple Silicon.
