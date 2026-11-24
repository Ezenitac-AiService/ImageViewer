# First Launch

**Language / 언어**: [한국어](FIRST_LAUNCH.md) | **[English](FIRST_LAUNCH.en.md)**

ImageViewer is not yet notarized with a paid Apple Developer certificate, so macOS Gatekeeper asks for confirmation **once**, the first time you open the download. You only do this once; after that it starts normally.

> Exact wording varies by macOS version. Windows is excluded under Constitution v3.0.0 (see [`legacy-v1.0.6`](https://github.com/Ezenitac-AiService/ImageViewer/tree/legacy-v1.0.6)), and mobile first-launch guides (`iOS` / `Android`) await feature `007`.

## macOS (one download for Apple Silicon and Intel)

1. Download and open `ImageViewer-macos-universal.dmg` from the [Releases page](https://github.com/Ezenitac-AiService/ImageViewer/releases). Apple Silicon and Intel Macs use the same file.
2. Drag **ImageViewer** onto the **Applications** folder in the window that opens.
3. Open ImageViewer from Applications. If a message like "Apple could not verify…" appears, click **Done** (or **Cancel**).
4. Go to **System Settings → Privacy & Security**, click **Open Anyway** next to the ImageViewer entry near the bottom, and confirm with your password or Touch ID.
5. Open it again; ImageViewer starts.

If macOS says the app **"is damaged and can't be opened"**, these steps will not help: the file was built incorrectly. Please report it as an [issue](https://github.com/Ezenitac-AiService/ImageViewer/issues).

## Linux (Lower-Priority Build-Only, Unverified)

Under Constitution v3.0.0, the Linux tarball is a **lower-priority build-only** release artifact and undergoes zero automated launch or runtime verification.

```bash
tar -xzf ImageViewer-linux-x64.tar.gz
./ImageViewer/imageviewer
```

It needs the GTK 3 (file dialogs) and libxkbcommon-x11 (X11 keyboard) runtimes, for example on Debian/Ubuntu: `sudo apt install libgtk-3-0 libxkbcommon-x11-0`.

## If it does not start

When ImageViewer cannot start, it shows a window explaining why and writes a diagnostic record here:

| OS | Diagnostic record folder |
|---|---|
| macOS | `~/Library/Application Support/com.Ezenitac-AiService.ImageViewer/logs` |
| Linux (unverified) | `~/.config/imageviewer/logs` |

When you file an issue, please paste the latest `imageviewer-YYYY-MM-DD.log` from that folder. Records contain file names only, never full folder paths.
