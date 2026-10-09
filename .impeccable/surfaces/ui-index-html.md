---
version: 1
slug: "ui-index-html"
primary_target: "ui/index.html"
related_targets: []
---

# Scambio main window

Scope: the single app window (ui/index.html), mode Operate. Audience: anyone with a Windows PC and an Android phone, often on a PC that is not theirs (exe on a USB stick). Job: send files to a nearby phone, accept files from it. Reference bar: Google Quick Share for Windows. Theme follows the Windows setting.

## Direction contract

THESIS: The category standard, executed straight: a native Windows 11 Fluent window with a radar of nearby devices. It refuses novelty; it must look like it shipped with Windows, and be cleaner than the official Quick Share app.

OWN-WORLD: Mica backdrop (solid Fluent neutral fallback on Windows 10), Segoe UI Variable type ramp (Caption 12, Body 14, Body Strong, Subtitle 20, Title 28), Windows accent blue #005FB8 light / #60CDFF dark, 4px control radius and 8px card radius, layer cards with 1px stroke, Fluent caption buttons, ToggleSwitch, ProgressBar, InfoBar, ContentDialog with smoke.

STORY: The user sees this PC at the centre of the radar and phones appearing around it, drops files, clicks a phone, compares the PIN, watches progress. Incoming requests interrupt as a ContentDialog with the PIN and Accept/Decline.

FIRST VIEWPORT: Custom 48px title bar (icon, "Scambio", minimize, close). Radar fills the top half: concentric rings, PC avatar in the centre with its name, device avatars (device-type glyph on a layer circle, name below; Quick Share advertises phone/tablet/laptop, and the glyph tells them apart at a glance better than an initial; the active device carries a determinate progress ring) placed on the rings; a sweeping pulse while searching. Below, each under a Settings-style group heading: the transfers list ("Attività") as layer cards, then the file drop card with "Scegli file" (accent button) and selected file list; bottom card "Ricezione" with visibility ToggleSwitch and download folder row. Primary action: click a device.

FORM: canon (the standing exit), chosen by the user; seed key bfa215af. Quality bar: Google Quick Share for Windows.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance
