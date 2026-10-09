---
name: Scambio
description: Portable Quick Share for Windows, drawn as a native Windows 11 Fluent window.
colors:
  accent: "#005fb8"
  accent-hover: "#1a6fc0"
  accent-press: "#3381c8"
  accent-tint: "rgba(0, 95, 184, 0.1)"
  on-accent: "#ffffff"
  accent-dark: "#60cdff"
  accent-hover-dark: "#5abce9"
  accent-press-dark: "#52a9d3"
  on-accent-dark: "#000000"
  bg-solid: "#f3f3f3"
  bg-solid-dark: "#202020"
  text: "rgba(0, 0, 0, 0.896)"
  text-2: "rgba(0, 0, 0, 0.606)"
  text-3: "rgba(0, 0, 0, 0.446)"
  text-dark: "#ffffff"
  text-2-dark: "rgba(255, 255, 255, 0.786)"
  text-3-dark: "rgba(255, 255, 255, 0.544)"
  card: "rgba(255, 255, 255, 0.7)"
  card-dark: "rgba(255, 255, 255, 0.0512)"
  card-stroke: "rgba(0, 0, 0, 0.0578)"
  control: "rgba(255, 255, 255, 0.7)"
  control-hover: "rgba(249, 249, 249, 0.5)"
  control-press: "rgba(249, 249, 249, 0.3)"
  control-stroke-bottom: "rgba(0, 0, 0, 0.1622)"
  subtle-hover: "rgba(0, 0, 0, 0.0373)"
  subtle-press: "rgba(0, 0, 0, 0.0241)"
  divider: "rgba(0, 0, 0, 0.0803)"
  dialog: "#ffffff"
  dialog-dark: "#2b2b2b"
  smoke: "rgba(0, 0, 0, 0.3)"
  critical: "#c42b1c"
  critical-bg: "#fde7e9"
  critical-dark: "#ff99a4"
  critical-bg-dark: "#442726"
  success: "#0f7b0f"
  success-dark: "#6ccb5f"
  caution: "#9d5d00"
  caution-bg: "#fff4ce"
  caution-dark: "#fce100"
  caution-bg-dark: "#433519"
typography:
  title:
    fontFamily: "Segoe UI Variable Display, Segoe UI Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "28px"
    fontWeight: 600
    lineHeight: "36px"
    letterSpacing: "0.12em"
    fontFeature: "tnum"
  subtitle:
    fontFamily: "Segoe UI Variable Display, Segoe UI Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "20px"
    fontWeight: 600
    lineHeight: "28px"
  body-strong:
    fontFamily: "Segoe UI Variable Text, Segoe UI Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 600
    lineHeight: "20px"
  body:
    fontFamily: "Segoe UI Variable Text, Segoe UI Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: "20px"
  caption:
    fontFamily: "Segoe UI Variable Text, Segoe UI Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: "16px"
rounded:
  control: "4px"
  card: "8px"
  toggle: "10px"
  full: "50%"
spacing:
  xxs: "4px"
  xs: "8px"
  sm: "12px"
  md: "16px"
  lg: "24px"
components:
  button-standard:
    backgroundColor: "{colors.control}"
    textColor: "{colors.text}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    padding: "5px 12px 6px"
    height: "32px"
  button-standard-hover:
    backgroundColor: "{colors.control-hover}"
  button-standard-active:
    backgroundColor: "{colors.control-press}"
    textColor: "{colors.text-2}"
  button-accent:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.on-accent}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    padding: "5px 12px 6px"
    height: "32px"
  button-accent-hover:
    backgroundColor: "{colors.accent-hover}"
  button-accent-active:
    backgroundColor: "{colors.accent-press}"
  button-subtle:
    backgroundColor: "transparent"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    height: "32px"
  button-subtle-hover:
    backgroundColor: "{colors.subtle-hover}"
  button-icon-only:
    rounded: "{rounded.control}"
    size: "32px"
  card:
    backgroundColor: "{colors.card}"
    rounded: "{rounded.card}"
    padding: "16px"
  infobar-caution:
    backgroundColor: "{colors.caution-bg}"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    padding: "12px 8px 12px 16px"
  infobar-critical:
    backgroundColor: "{colors.critical-bg}"
    textColor: "{colors.text}"
    rounded: "{rounded.control}"
    padding: "12px 8px 12px 16px"
  dialog:
    backgroundColor: "{colors.dialog}"
    textColor: "{colors.text}"
    rounded: "{rounded.card}"
    padding: "24px"
    width: "400px"
  toggle-on:
    backgroundColor: "{colors.accent}"
    rounded: "{rounded.toggle}"
    width: "40px"
    height: "20px"
  toggle-off:
    backgroundColor: "{colors.control-press}"
    rounded: "{rounded.toggle}"
    width: "40px"
    height: "20px"
  pin-block:
    backgroundColor: "{colors.accent-tint}"
    typography: "{typography.title}"
    rounded: "{rounded.control}"
    padding: "12px 16px"
---

# Design System: Scambio

## Overview

**Creative North Star: "Shipped With Windows"**

Scambio is the category standard executed straight: a single small Windows 11 window that should read as if it came in the box. Every surface uses the Fluent canon (Mica backdrop, Segoe UI Variable, the system accent, layer cards with hairline strokes, Fluent controls) and the bar is to be cleaner than Google's own Quick Share for Windows. Novelty is refused on purpose; the only bespoke element is the radar, and even it is drawn with Fluent materials.

The window is a single scrolling column: a 48px custom title bar, the radar of nearby devices on top, then Settings-style groups (transfers, files to send, receiving) as layer cards. Density is desktop-comfortable, not dense: 12px between groups, 8px from a group heading to its card, 16px card padding. Light and dark follow the system setting, and Mica is painted by Windows behind a transparent page with a solid neutral fallback when Mica is unavailable.

Motion is Fluent's: 83ms linear for hover and press fills, 167ms for fades and toggle travel, 250-367ms decelerate (`cubic-bezier(0, 0, 0, 1)`) for things entering. Under reduced motion everything collapses to 1ms, the radar pulse disappears and indeterminate progress becomes a static dimmed bar.

**Key Characteristics:**
- Native Windows 11 Fluent canon, light and dark from the system theme.
- Mica backdrop with solid fallback; translucent layer cards with 1px strokes.
- One accent (the Windows accent blue) for primary action, selection, progress and the radar's own PC.
- Segoe UI Variable ramp: Caption 12, Body 14, Body Strong 14, Subtitle 20, Title 28.
- Segoe Fluent Icons for every glyph, the platform's own icon font.
- Radar of nearby devices as the signature component.

## Colors

The Windows neutral palette with one accent and the system status colors; every value has a light and a dark counterpart switched by `prefers-color-scheme`.

### Primary
- **Windows Accent Blue** (accent / accent-dark): the accent button, the toggle's on state, progress fills, the busy ring around a device, the PC avatar at the radar centre, device glyphs, the drop-overlay frame and selection. Hover and press step lighter in light mode and darker in dark mode, exactly as Fluent does.
- **Accent Wash** (accent-tint): a 10-14% accent fill behind the PIN block and under the drag-and-drop overlay. It marks "this is the thing to compare or target" without a second hue.

### Neutral
- **Mica Fallback** (bg-solid / bg-solid-dark): the window background when Mica is not available, and the dialog footer.
- **Text ladder** (text, text-2, text-3): primary text, secondary captions and statuses, disabled text and the progress rail.
- **Layer Card** (card / card-dark): translucent white over Mica in light mode, a faint white lift in dark mode.
- **Control fills** (control, control-hover, control-press): standard buttons and the toggle's off track.
- **Subtle fills** (subtle-hover, subtle-press): hover and press for transparent controls, list rows, caption buttons and the inline text-payload box.
- **Hairlines** (card-stroke, control-stroke-bottom, divider): 1px card and dialog strokes, the heavier bottom edge that gives Fluent controls their depth, and dividers between setting rows.
- **Dialog Surface** (dialog / dialog-dark) over **Smoke** (smoke): the ContentDialog sheet and its 30% black scrim.

### Status
- **Critical** (critical, critical-bg): failed transfers and the error InfoBar. The caption close button uses the same red on hover in both themes, as Windows does.
- **Success** (success): the completed-transfer glyph only.
- **Caution** (caution, caution-bg): the "no devices found" help InfoBar.

### Named Rules
**The One Accent Rule.** The Windows accent blue is the only hue that is not a status color. No secondary brand color exists; if something needs emphasis, it gets the accent or Body Strong, never a new hue.

**The Status Means Status Rule.** Red, green and amber appear only on InfoBars and transfer outcome glyphs. They are never decoration.

## Typography

**Display Font:** Segoe UI Variable Display (with Segoe UI, system-ui)
**Body Font:** Segoe UI Variable Text (with Segoe UI, system-ui)
**Icon Font:** Segoe Fluent Icons (with Segoe MDL2 Assets)

**Character:** The Windows system face at its optical sizes: Text for everything 14px and below, Display for 20px and up. It is the platform voice, not a choice to be admired.

### Hierarchy
- **Title** (600, 28px/36px, 0.12em tracking, tabular figures): the PIN code in the incoming-transfer dialog only.
- **Subtitle** (600, 20px/28px, Display): the dialog title and the drop-overlay message.
- **Body Strong** (600, 14px/20px): group headings ("Attività", "File da inviare", "Ricezione"), InfoBar titles, emphasised words and the inline PIN.
- **Body** (400, 14px/20px): the base for all controls, file names, transfer titles and setting labels.
- **Caption** (400, 12px/16px; 600 for this PC's name under the radar centre): the title-bar title, device names, the radar status line, secondary paths and statuses (usually text-2).

### Named Rules
**The Fluent Ramp Rule.** Only the five ramp steps above exist. Hierarchy comes from size and weight steps of the ramp, never from uppercase, letter-spaced labels or colored text; the one tracked style is the PIN, where spacing serves digit comparison.

## Layout

A single column inside a small desktop window. The title bar is 48px tall with 16px left inset and Fluent caption buttons 46px wide. Content scrolls under it with 16px side and bottom padding, a stable 6px overlay scrollbar that appears only on hover, and 12px between groups.

The radar is a square stage up to 340px wide (`min(100%, 340px)`) and a size container: devices are positioned on it in container-relative units, so the composition scales with the window. Below it, each group is a Settings-style heading (inset 4px) followed by its card at 8px.

Spacing runs on a 4px base: 4, 8, 12, 16, 24. 4px for tight internal gaps, 8px between related controls, 12px between an icon and its text and between groups, 16px card padding, 24px dialog padding. Setting rows are at least 48px tall; buttons are 32px.

## Elevation & Depth

Depth is Fluent's: mostly material and stroke, with very soft ambient shadows. Layer cards sit on Mica with a 1px stroke and a barely visible 2px shadow; controls get their lift from a heavier bottom stroke rather than a shadow. Only the ContentDialog floats, on a large diffuse two-layer shadow over a smoke scrim. The radar's avatars carry small ambient shadows so they read as objects on the stage.

### Shadow Vocabulary
- **Card** (`box-shadow: var(--shadow-card)`, light `0 2px 4px rgba(0,0,0,0.04)`, dark `0 2px 4px rgba(0,0,0,0.13)`): every layer card.
- **Dialog** (`box-shadow: var(--shadow-dialog)`, light `0 32px 64px rgba(0,0,0,0.1876), 0 2px 21px rgba(0,0,0,0.1474)`): the ContentDialog only.
- **Device avatar** (`box-shadow: 0 2px 6px rgba(0,0,0,0.08)`): device circles on the radar.
- **PC avatar** (`box-shadow: 0 4px 12px rgba(0,0,0,0.14)`): the accent circle at the radar centre.

### Named Rules
**The Stroke Before Shadow Rule.** Separation comes first from the 1px stroke and the Mica/layer contrast. Shadows stay ambient and soft; only the dialog casts a real one.

## Shapes

Two radii carry the whole system: 4px for controls (buttons, InfoBars, list rows, the drop target, the PIN block) and 8px for containers (cards, dialog, the drop overlay, the device hit area). The close caption button rounds only its top-right corner (8px) to follow the window corner. Avatars, rings and the radar pulse are full circles; the toggle is a 10px-radius pill with a 12px circular thumb. Dashed strokes appear only where a drop is invited: the empty drop target, the drag overlay, and the outer radar ring that marks the search horizon.

## Components

### Buttons
Fluent buttons, exact to the platform.
- **Shape:** gently rounded (4px), 32px minimum height, padding 5px 12px 6px, 8px gap to an icon.
- **Standard:** control fill, 1px control stroke with a heavier bottom stroke; hover lightens the fill, press dims the text to text-2 and flattens the bottom stroke.
- **Accent:** accent fill, on-accent text, a faint white stroke and a 40% black bottom edge; one per view, for the primary action ("Scegli file", "Accetta").
- **Subtle:** transparent until hover (subtle-hover); used for secondary actions in a row ("Svuota", "Annulla") and for icon-only dismiss buttons (32px square, 12px glyph).
- **Transitions:** background 83ms linear. Focus is the system ring (2px text-colored outline, 1px offset, inner bg-colored halo).

### Cards / Containers
- **Corner Style:** 8px.
- **Background:** card layer; dialog surface when Mica is off where a solid face is needed.
- **Shadow Strategy:** Card shadow (see Elevation).
- **Border:** 1px card-stroke.
- **Internal Padding:** 16px (files card), 8px 16px (settings card), 12px 12px 12px 16px (transfer card).

### Settings rows
A Windows Settings row: 20px-wide glyph, label in Body with a Caption description below, action on the right (ToggleSwitch or standard buttons), at least 48px tall, 1px divider between rows. Long paths truncate from the start so the folder name stays visible.

### ToggleSwitch
40 by 20px pill. Off: 1px strong stroke, control-press fill, 12px dark thumb. On: accent track and an on-accent thumb that slides 20px in 167ms decelerate. Hover grows the thumb to 14px. A text label ("Attivato") sits 12px to the right.

### InfoBar
4px radius, 1px stroke, caution or critical background, status-colored glyph, Body Strong title, a bulleted fix list in text-2, and a subtle icon-only close button. It fades in over 167ms. Failures always arrive here in plain words with the fix.

### ProgressBar
3px tall, a 1px text-3 rail with a full-height accent fill, 2px rounded ends. Determinate fill scales in 250ms decelerate; indeterminate runs a 40% segment across in 1.8s.

### ContentDialog
Max 400px, 8px radius, dialog surface, Dialog shadow over a 30% smoke scrim, entering with a 250ms scale-down from 1.05. 24px body with Subtitle heading; a footer in bg-solid with a top hairline and two equal-width buttons (accent first). The incoming-transfer dialog carries the PIN block: accent-tint fill, 4px radius, "PIN" caption label and the Title-sized code.

### Radar (signature)
A square stage with three concentric 1px rings (36%, 66%, 96%; the outer one dashed) and two staggered accent pulses that expand over 3.2s while searching and stop when idle. This PC sits at the centre as a 64px accent avatar with its name in Caption at 600. Nearby devices are 52px layer circles with an accent device-type glyph (phone, tablet, laptop) and a Caption name, entering in 367ms with a scale-up and leaving in 167ms. A busy device wears a 2px accent ring: a sweeping arc while waiting, a determinate conic ring once progress is known. The whole device is a subtle button (8px radius) that presses the avatar to 0.96.

### Drop target
In the files card: a dashed 1px control-stroke-bottom frame, 4px radius, centred accent glyph, a one-line instruction and the accent button. When files are dragged over the window, a full-window overlay appears inset 8px below the title bar: 2px dashed accent frame, accent wash over the solid background, a 40px glyph and a Subtitle message.

## Do's and Don'ts

### Do:
- **Do** take every color from the custom properties and make sure each has its dark counterpart in the `prefers-color-scheme: dark` block.
- **Do** keep the page transparent when Mica is on and fall back to bg-solid (#f3f3f3 / #202020) when it is not.
- **Do** use 4px for controls and 8px for containers, nothing else except circles and the toggle pill.
- **Do** use Segoe Fluent Icons for every glyph, at 16px by default (12px in icon-only buttons, 10px in caption buttons).
- **Do** use the Fluent timings: 83ms linear for fills, 167ms for fades and toggles, `cubic-bezier(0, 0, 0, 1)` for entrances; honour reduced motion by removing the pulse and freezing indeterminate progress.
- **Do** keep a single accent button per view, for the primary action.

### Don't:
- **Don't** introduce a second accent hue, gradients or brand colors; the Windows accent is the brand.
- **Don't** add type sizes outside the Caption/Body/Subtitle/Title ramp, or uppercase tracked labels.
- **Don't** add heavy or hard-edged shadows; only the ContentDialog casts a large shadow.
- **Don't** draw icons as images, emoji or another icon set next to Segoe Fluent Icons.
- **Don't** use status colors outside InfoBars and transfer outcome glyphs.
