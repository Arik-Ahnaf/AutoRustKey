# AutoHotkey Dash (UX) — mechanics sheet

The original program `AutoHotkeyUX.exe` is the stock AutoHotkey v2.0.19 64-bit
interpreter (version resource "AutoHotkey 64-bit 2.0.19"; the string
"AutoHotkey v2.0.19" is used by function 0x140045e00 when it builds the script
title). Its hash equals `v2\AutoHotkey64.exe` in `installed-files.csv`.
Everything the person sees is defined by the `.ahk` scripts next to it, so the
rules below cite those script lines (`file:line`). This Rust rebuild is
written from this sheet, not from the scripts' code.

## Common look (inc/ui-base.ahk)
| Value | Setting | Source |
|---|---|---|
| Font | Segoe UI 9 pt (12 px at 96 DPI) — on Linux the closest system sans (Noto Sans / DejaVu) | ui-base.ahk:4 |
| Background | system COLOR_3DFACE = #F0F0F0 | ui-base.ahk:67 |
| Margins | X 10 px, Y 6 px (AutoHotkey defaults for 9 pt; measured from capture) | capture of Dash |
| Escape / close | closes the window | ui-base.ahk:5-6 |
| List menu | single-select Tile view, no border, Explorer hover theme, label margin 2 px | ui-base.ahk:9-33 |
| Tile list auto-size | height fits min(count, maxItems) tiles + 4 px; maxItems 20 (Dash), 8 (New Script, Editor) | ui-base.ahk:103-128 |
| Single instance per window class | opening a window that is already open shows the existing one | ui-base.ahk:53-59 |

## Dash (ui-dash.ahk)
* Title "AutoHotkey Dash", list 250 px wide (ui-dash.ahk:21), 32 px icons.
* Items (ui-dash.ahk:34-45): New script / Compile / Help files (F1) / Window spy /
  Launch settings / Editor settings, each with the description line shown.
* Click (single) or Enter activates; F1 anywhere opens help (ui-dash.ahk:93-133).
* Intro panel shown unless config `Dash\SuppressIntro` is true (ui-dash.ahk:54):
  "Welcome!" at 12 pt, two lines of text, six help links, checkbox
  "Show this info next time" aligned with the list bottom; unticking writes
  SuppressIntro=1, ticking deletes it (ui-dash.ahk:86-91).
* Help links open `<root>/v2/AutoHotkey.chm::docs/<page>` if the CHM exists,
  else `https://www.autohotkey.com/docs/v2/<page>` (ui-dash.ahk:79-84).
* Window height = list height + 2 × MarginY (ui-dash.ahk:76).
* Compile: run `<root>/Compiler/Ahk2Exe.exe` if present (Linux: through Wine),
  otherwise offer to get Ahk2Exe (ui-dash.ahk:109-115, install-ahk2exe.ahk).
* Help menu (ui-dash.ahk:141-198): offline `.chm` files grouped by major
  version, newest per major first, others under "More"; then "Online help"
  with v1.1 and v2.0 links; then "Other files". If exactly one AutoHotkey
  CHM and no other CHM exists, it opens directly.

## New Script (ui-newscript.ahk)
* Name box 272 px with cue "Untitled", browse button 28 px; folder box 300 px
  read-only; template list 300 px, max 8 visible (ui-newscript.ahk:22-47).
* Default folder: config `New\DefaultDir`, else `<Documents>/AutoHotkey` (:20).
* Templates: "Empty — Clean slate", then every `*.ahk` in `<UX>/Templates` and
  `<Documents>/AutoHotkey/Templates` (user overrides same name), description
  from the `[NewScriptTemplate] Description=` header, `Execute=true` runs it
  instead of copying (:200-258). Hidden ones listed in `New\HideTemplate`.
* Default template `New\DefaultTemplate` is pre-selected (:39-45).
* Buttons: Defaults (icon, 28 px), Create (75 px at x150), Edit (75 px at
  xm+225). Default button = `New\DefaultButton` (Edit unless "Create") (:51-57).
* Confirm (:126-163): needs a selected template ("You need to select a template
  first."); file name defaults to "Untitled", strips ".ahk", adds "-1", "-2"…
  until unused; writes the template with the `/* [NewScriptTemplate] … */`
  block removed, as UTF-8; Create shows the file in the file manager, Edit
  opens it in the editor; window stays open if Ctrl is held or
  `New\StayOpen` is set.
* Defaults menu (:92-124): Default to Create / Default to Edit (radio),
  Stay open (check), Set folder as default (check if equal), Open templates folder.
* Template right-click (:165-186): Edit template (copies to user folder
  first), Hide template, Set as default (checked if current).
* Up/Down in the name box move the list selection (:67-80).

## Launch Config (ui-launcherconfig.ahk)
* Radios "Auto-detect version when launching script" / "Run all scripts with a
  specific interpreter" (:43-46).
* Auto-detect page: "Preferred interpreter by major version"; v1 row:
  DDL 110 px ["Latest 1.x", versions…], build combo 150 px ["Unicode 64-bit",
  "Unicode 32-bit", "ANSI 32-bit"], UI Access; v2 row: ["Latest 2.x", …],
  ["64-bit","32-bit"], UI Access; "When detection fails" DDL
  ["Use v1.x","Use v2.x","Ask the user"] default 3; checkboxes "Try to identify
  version based on syntax" (default on) and "Default to UTF-8 even for v1
  scripts" (default off) (:50-70).
* Specific page: read-only interpreter path 326 px + browse 28 px, "Default to
  UTF-8" only for v1 interpreters (:73-84, :126-139).
* Close button 70 px at x292; window height follows the visible page (:115-124).
* Config keys: `Launcher\v1|v2\Version` (blank = latest), `\Build`, `\UIA`,
  `Launcher\Fallback` (blank = ask), `Launcher\Identify`, `Launcher\v1\UTF8`.
* Applying writes the open/run command for .ahk files (:162-191). Linux: a
  `.desktop` entry + MIME type `text/x-autohotkey` set as default with xdg-mime.

## Launcher (launcher.ahk, launcher-common.ahk, identify.ahk)
* `#Requires AutoHotkey <req> [; prefer <p>]` decides the version (:65-70).
* Else, if Identify is on, syntax detection counts v1-only vs v2-only marks;
  picks a version only if exactly one kind is found (identify.ahk:19-28).
  Rebuild uses a simplified marker list (estimated, see below).
* No version → Fallback; still none → ask the user (VersionSelectGui
  "Run <script> with", 200 px list) (:85-99, :258-282).
* Interpreter choice: requirements filter, then a bit-score over
  [configured version, prefer list, build, UIA] — earlier criteria weigh more;
  ties prefer the later version (:195-256). Default prefer "!UIA, 64, !ANSI".
* v1 + `Launcher\v1\UTF8` adds `/CP65001` (:104-107).
* Usable interpreters come from `UX/installed-files.csv` rows whose
  Description matches `^AutoHotkey.* (32|64)-bit` and that aren't
  AutoHotkeyUX.exe; otherwise a recursive search for AutoHotkey*.exe
  (launcher-common.ahk:24-78). Linux runs them with `wine`.

## Editor (ui-editor.ahk)
* Title "Select an editor", list 300 px, max 8 tiles, "Command line" 2-row
  edit, info frame with link "Editors with AutoHotkey support", Browse / OK /
  Cancel 80 px each (:14-40).
* OK is enabled only when the command's first word is an existing program (:87-92).
* Picking a list item fills the command; typing deselects (:82-92).
* Commands without %1/%l get ` "%l"` appended (:172-173).
* Saved as the .ahk "edit" command (:146-150). Linux: config `Editor\Command`
  and editors come from `.desktop` files that handle text/plain.

## Window Spy (WindowSpy.ahk)
* Title "Window Spy for AHKv2", always on top, resizable, fields 320 px wide;
  updates every 250 ms; Ctrl/Shift held suspends; hovering own window shows
  "(Updates suspended)" (:22-56, :77-108).
* Text formats: `ahk_class`, `ahk_exe`, `ahk_pid`, `ahk_id`; "Screen:\t x, y",
  colour "RRGGBB (Red=RR Green=GG Blue=BB)"; text over 40 chars or multi-line
  is cut with " (...)" (:111-234).
* Linux: data comes from xdotool/xprop (X11) or kdotool (KDE Wayland). Status
  bar / visible text have no Linux equivalent and stay empty.
