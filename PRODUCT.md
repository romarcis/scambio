# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Tauri 2 (Rust backend + WebView2), UI in plain HTML/CSS/JS, no framework. Ships as a single portable `.exe` (no installer). Quick Share protocol from the vendored `core/` crate (fork of rquickshare `core_lib`, GPL-3.0).

## Users

People with a Windows PC and an Android phone who want to move files between them without cables or cloud. The exe lives on a USB stick and runs on PCs that are not theirs, so a first-time user must understand it at a glance, with no setup.

## Product Purpose

Scambio sends files from a Windows PC to nearby Android phones over Quick Share, and receives files the phones send. Success: the phone shows up, the file arrives, nothing to install.

## Positioning

Portable Quick Share for Windows: one exe, runs from a USB stick, no Google account or installation. The official Google app needs an installer.

## Operating Context

- PC and phone on the same Wi-Fi network; Bluetooth on the PC helps wake the phones (BLE beacon) but is optional.
- Sending: the user picks files (drag & drop or a file picker), the phones nearby appear, the user picks one, the phone shows a confirmation with a PIN, the transfer runs with progress.
- Receiving: the PC is visible to the phones; an incoming request shows the sender, the files and the PIN, and the user accepts or declines. Files land in a download folder.
- Windows Firewall may ask for permission on first run (mDNS + TCP).

## Capabilities and Constraints

- Discovery works only for phones on the same LAN with Quick Share set to be visible (or woken by the BLE beacon).
- Transfers: files and text/URLs. Cancelation possible from both sides.
- Small desktop window; Italian UI.
- Undecided: default download folder when running from USB (next to the exe vs the user's Downloads folder).

## Brand Commitments

Name: Scambio. No logo or assets exist yet.

## Evidence on Hand

None. No real device names, screenshots or user data; do not invent claims about speed or compatibility.

## Product Principles

1. Zero setup: double-click and it works, on any PC.
2. Always show what is happening on the network (searching, found, waiting for the phone, transferring), never a silent wait.
3. The person decides: incoming files are never accepted without consent.
4. Explain failures in plain words with the fix (same Wi-Fi, firewall, visibility on the phone).

## Accessibility & Inclusion

Full keyboard operation; readable by people who do not know Quick Share.
