# Scambio

Scambio is a portable Windows application that exchanges files with nearby Android devices over Google's Quick Share protocol (formerly Nearby Share). It runs as a single executable, requires no installation and no Google account, and can therefore be carried on a USB drive and launched on any compatible computer.

## Features

- **Discovery of nearby devices.** Android phones and tablets that are visible over Quick Share on the local network are listed as they are found.
- **Sending files.** Files may be selected through a file picker or dropped onto the window, and are sent to the chosen device after the recipient accepts the transfer.
- **Receiving files.** The computer is advertised as a Quick Share destination. Each incoming request shows the sender, the files and a confirmation PIN, and nothing is saved until the user accepts it.
- **Progress and status.** Every transfer reports its current stage (connection, confirmation, transfer, completion) together with its progress.
- **Portability.** The application consists of a single executable file. The only data written outside the download folder is the WebView2 cache, which Windows keeps in the user's local application data folder.

## Requirements

| Requirement | Details |
|---|---|
| Operating system | Windows 10 or Windows 11, 64-bit |
| Runtime | Microsoft Edge WebView2, which is preinstalled on Windows 11 and on current Windows 10 builds |
| Network | The computer and the Android device must be connected to the same local network |
| Bluetooth (optional) | When a Bluetooth adapter is present, Scambio emits a Quick Share beacon that prompts nearby Android devices to make themselves discoverable |

## Usage

1. Launch `Scambio.exe`. On first launch, Windows Defender Firewall may ask whether to allow network access. Access to private networks is required for discovery and transfers.
2. **To send:** add the files, wait for the target device to appear in the list, and select it. Confirm that the PIN shown by Scambio matches the PIN shown on the phone, then accept the transfer on the phone.
3. **To receive:** keep Scambio open and visible. On the Android device, open the share sheet, choose Quick Share, and select the computer by name. Compare the PIN, then accept or decline the request in Scambio. Received files are saved to the download folder, which defaults to the user's *Downloads* folder and can be changed from within the application.

### Troubleshooting

- **The device does not appear.** Make sure both devices are on the same network, that Quick Share on the phone is set to be visible to *Everyone* or that its sharing screen is open, and that the firewall permits Scambio on the current network profile. Networks that isolate clients from one another, such as many guest and public networks, prevent discovery entirely.
- **The computer does not appear on the phone.** Make sure receiving is enabled in Scambio and that the firewall permits incoming connections.

## Building from source

The application is built with [Tauri 2](https://tauri.app/) and Rust. The user interface consists of static HTML, CSS and JavaScript and needs no build step.

Prerequisites: a stable Rust toolchain (MSVC target) and Node.js, which is used only to run the Tauri command-line interface.

```sh
# Run in development
npx @tauri-apps/cli@2 dev

# Produce the portable executable
npx @tauri-apps/cli@2 build --no-bundle
```

The resulting executable is located at `target/release/scambio.exe`.

## Project structure

| Path | Contents |
|---|---|
| `core/` | Quick Share protocol implementation: mDNS discovery and advertisement, UKEY2 key exchange, encrypted payload transfer, and the Windows Bluetooth LE beacon |
| `src-tauri/` | Application backend, which connects the protocol library to the user interface |
| `ui/` | User interface (HTML, CSS, JavaScript) |

## Acknowledgements

The protocol implementation in `core/` is derived from the `core_lib` crate of [RQuickShare](https://github.com/Martichou/rquickshare) by Martin André. It has been adapted to run on Windows: the Linux-specific Bluetooth advertiser has been replaced with one based on the Windows Runtime, Unix-specific file operations have been made portable, and the Protocol Buffers compiler is now vendored. The protocol itself was documented by the [NearDrop](https://github.com/grishka/NearDrop) project.

Quick Share is a trademark of Google LLC. Scambio is an independent project and is neither affiliated with nor endorsed by Google.

## License

Scambio is distributed under the terms of the GNU General Public License, version 3. See [LICENSE](LICENSE) for the full text.
