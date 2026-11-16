# Running the acceptance check on any Mac

These steps are for a person with the disk image and nothing else: no developer tools, no account. The check uses a throwaway Capsule. It writes nothing to the keychain and deletes that Capsule when it ends.

The Mac needs macOS 13.3 or later. It can be Intel or Apple Silicon.

## 1. Put the app in Applications

1. Double-click `Isonomic-<version>-<build>.dmg`.
2. In the window that opens, drag **Isonomic** onto **Applications**.
3. Eject the disk image (the eject arrow beside it in Finder's sidebar).

The check can't run from the disk image itself. macOS keeps an app on a disk image marked as downloaded and unopened, and that mark can't be cleared there.

## 2. Let macOS run it

Isonomic isn't signed with an Apple developer identity yet, so macOS refuses to run it until told otherwise.

For the check, **don't open Isonomic from Finder**. Opening it starts a real first run, with a real Capsule in the person's Library (and, on a Mac with enough memory, a real Isonomic ID in the keychain). Instead, open **Terminal** (Applications › Utilities › Terminal), paste this line and press Return:

```
xattr -dr com.apple.quarantine /Applications/Isonomic.app
```

It clears the "downloaded from the internet" mark from the app. This is what **Open Anyway** in Privacy & Security does, but nothing starts, and no Capsule is made. It prints nothing when it works.

(To use Isonomic normally later, follow "Opening Isonomic the first time" on the disk image instead.)

## 3. Run the check

In the same Terminal window, paste this line and press Return:

```
sh /Applications/Isonomic.app/Contents/Resources/acceptance-check
```

The Isonomic window opens and drives itself: first run, a question to Chat, stop and continue, then a second launch. Leave the Mac alone until Terminal prints a last line that starts `acceptance check:`.

- **The first time, it downloads the model, which takes a while.** The model is gigabytes, and first run shows its size; the check allows up to 30 minutes for the download. The download goes into the throwaway Capsule, so it's deleted at the end too: each run downloads again.
- **If macOS asks** whether Terminal may control or access something, choose **Don't Allow**. The check needs none of it.

## 4. Copy back

Select everything Terminal printed from the `sh …` line down, with **Edit › Select All**, then copy it and send it. The first line names the macOS version, the chip, the memory and the app's build; no names or paths. A failed run also prints the last lines of the engine's and the window's logs, with the home folder shown as `~`.

## On a Mac with 8 GB of memory

V1 runs local models only, and needs at least 16 GB of memory: the window and the engine both check this.

- **On an 8 GB Mac, first run says** "8.0 GB of memory. Isonomic needs at least 16.0 GB to run its local models."
- **First run offers no model,** downloads nothing, makes no Isonomic ID, and Continue stays off.
- **The check stops there.** It prints:

  ```
  [self-check] … FAIL: first run: this Mac has 8.0 GB of memory; Isonomic needs at least 16.0 GB, so first run offers no model
  acceptance check: FAIL (prompt failed, link failed)
  ```

  followed by the logs' last lines. That run still shows the app installs, opens on Apple Silicon, and refuses plainly.

## What the check leaves behind

- **Deleted when it ends:** the Capsule, in the temporary folder.
- **Left in place:** the copy of Isonomic in Applications. Drag it to the Bin to remove it.
- **Possibly kept by macOS:** its own records of having run the app, such as the window's saved position. They hold nothing of Isonomic's.

## The full pass

- **The release app runs the check described above:** first run, a Chat turn, stop and continue, keys in memory, and the relaunch through a link.
- **The full pass needs the check build** (`cargo xtask bundle --check-build`). It is never a release. The full pass's later steps and its screenshots run only in that build, and the same script runs them there. It saves the screenshots in a folder on the Desktop.
