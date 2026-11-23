# First Launch

**Language / 언어**: [한국어](FIRST_LAUNCH.md) | **[English](FIRST_LAUNCH.en.md)**

ImageViewer is not yet signed with a paid publisher certificate, so the operating system asks for confirmation **once**, the first time you open the download. You only do this once; after that it starts normally.

> The exact wording and look of the confirmation depend on the OS version. Screenshots from the pre-release clean-machine check will be added here (task T110).

## Windows 10 / 11

1. Download `ImageViewer-windows-x64.exe` from the [Releases page](https://github.com/Ezenitac-AiService/ImageViewer/releases). It is a single file that runs without installation.
2. When you run it, **"Windows protected your PC"** may appear.
3. Click **More info**, then **Run anyway**.
4. The ImageViewer window opens. No separate black console window appears.

Nothing else needs to be installed (no Visual C++ redistributable).

On PCs with **Smart App Control** turned on, unsigned apps cannot be run at all. Please wait for a publisher-signed release in that case.

## macOS (one download for Apple Silicon and Intel)

1. Download and open `ImageViewer-macos-universal.dmg`. Apple Silicon and Intel Macs use the same file.
2. Drag **ImageViewer** onto the **Applications** folder in the window that opens.
3. Open ImageViewer from Applications. If a message like "Apple could not verify…" appears, click **Done** (or **Cancel**).
4. Go to **System Settings → Privacy & Security**, click **Open Anyway** next to the ImageViewer entry near the bottom, and confirm with your password or Touch ID.
5. Open it again; ImageViewer starts.

If macOS says the app **"is damaged and can't be opened"**, these steps will not help: the file was built incorrectly. Please report it as an [issue](https://github.com/Ezenitac-AiService/ImageViewer/issues).

## Linux (experimental)

The Linux build passes the automated checks but has not passed launch verification; it is **experimental**.

```bash
tar -xzf ImageViewer-linux-x64.tar.gz
./ImageViewer/imageviewer
```

It needs the GTK 3 (file dialogs) and libxkbcommon-x11 (X11 keyboard) runtimes, for example on Debian/Ubuntu: `sudo apt install libgtk-3-0 libxkbcommon-x11-0`.

## If it does not start

When ImageViewer cannot start, it shows a window explaining why and writes a diagnostic record here:

| OS | Diagnostic record folder |
|---|---|
| Windows | `%LOCALAPPDATA%\ImageViewer\logs` |
| macOS | `~/Library/Application Support/com.Ezenitac-AiService.ImageViewer/logs` |
| Linux | `~/.config/imageviewer/logs` |

When you file an issue, please paste the latest `imageviewer-YYYY-MM-DD.log` from that folder. Records contain file names only, never full folder paths.
