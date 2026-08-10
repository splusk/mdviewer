# mdviewer

A terminal-based Markdown viewer written in Rust. Renders Markdown files with syntax highlighting, styled formatting, and interactive navigation.

## Acknowledgements

**mdviewer is a fork of [bahdotsh/mdterm](https://github.com/bahdotsh/mdterm), created by
[@bahdotsh](https://github.com/bahdotsh).** Essentially everything that makes this a good
Markdown viewer is their work: the renderer, the TUI, syntax highlighting, image protocols,
mermaid diagrams, math rendering, the JSON viewer, search, table of contents, slide mode,
themes, and auto-reload. Full credit for the original goes to them, and the
[Features](#features) list below is overwhelmingly a description of *their* work.

It also incorporates the searchable file picker from
[mdterm#60](https://github.com/bahdotsh/mdterm/pull/60) by
[@seobaeksol](https://github.com/seobaeksol), which was still an open pull request upstream
at the time of forking.

This fork exists to scratch a specific itch — reading an [Obsidian](https://obsidian.md)
vault in the terminal — and adds only the following on top:

| Addition | What it does |
|----------|--------------|
| `[hide] images` / `--no-images` | Skips images entirely: no placeholder rows, no caption, and no network fetch. Useful where terminal image protocols don't work well (e.g. some tmux setups) |
| `[hide] images_in_links` | Skips only images nested inside a link's label (e.g. a decorative icon prefixing a link) — they'd otherwise break the link's line into three. Standalone images are unaffected. Config-only, no CLI flag |
| `[hide] frontmatter` / `--no-frontmatter` | Drops a leading `---` YAML block, which otherwise renders as two rules with raw YAML between them |
| `[hide] code_languages` / `--hide-code-lang` | Omits fenced blocks by language, e.g. Obsidian's ```` ```dataviewjs ```` blocks, which mean nothing outside Obsidian |
| `[picker] ignore` | Skips files and directories by name in the file picker |
| `[picker] hidden` | Dot-directories (`.obsidian`, `.git`, `.trash`) are now skipped by default, matching `fd` |
| `[picker] max_results` | Caps how many results the picker lists at once, without limiting what can be found by typing |
| `[picker] sort_by` | Sorts by last-modified time (`"mtime,desc"` / `"mtime,asc"`) instead of alphabetically |
| Picker rooting | Pressing `p` roots the picker at the directory mdviewer was launched from, rather than the open file's folder |
| Picker key fix | `p` and `q` are literal search characters in the picker; previously they closed it — or quit outright — so no query containing them could be typed |
| Picker query reset | The picker's search text no longer persists across close/reopen — previously it kept whatever was last typed |
| Config lookup | Searches `$XDG_CONFIG_HOME`, then `~/.config`, then the platform directory, so the documented `~/.config/…/config.toml` works on macOS too (see [mdterm#66](https://github.com/bahdotsh/mdterm/issues/66)) |
| Wikilinks | Rewrites Obsidian-style `[[note]]` links and `![[image.png]]` / `![[file.pdf]]` embeds into standard Markdown before parsing, so they render and navigate like ordinary links/images. Tolerates a literal `]` inside the target (e.g. `![[Offer Letter [Name].pdf]]`), which previously broke the regex and left the whole embed as unrendered raw text |
| Wikilink embed resolution | `![[photo.png]]` resolves by searching the note's own directory first, then its ancestors (nearest first) — one shared `attachments/` folder anywhere above a note resolves for every note nested under it, no path needed in the embed itself |
| `attachment_folder_path` | Extra subfolder name (e.g. `"attachments"`) checked at each level of that ancestor search — for vaults that keep a dedicated attachments folder *alongside* each note rather than shared above them |
| Image path fix | Local image paths resolve relative to the markdown file's own directory, not mdviewer's current working directory |
| Quick Look preview | `Enter` or click on a wikilink embed's caption opens the real file at full quality via the OS's native previewer (`qlmanage`/`open`) instead of a "Blocked: unsupported URL scheme" toast |
| Non-image embed captions | A hidden or non-image wikilink embed (e.g. an attached PDF) shows its caption/link instead of leaving a block of dead space where a stuck `[ Loading: ... ]` placeholder used to sit forever |
| iTerm2 tmux fix | Chunks the inline-image escape sequence for iTerm2 under tmux, fixing images that silently failed to render when the sequence exceeded tmux's passthrough buffer |
| Mermaid sequence diagrams | `sequenceDiagram` blocks get a real parser and renderer — participants, all arrow variants, activation shorthand, notes, nested `loop`/`alt`/`opt`/`par` — instead of being fed to the flowchart-only parser and rendering as scrambled one-word boxes |
| `Shift+P` | Copies the current file's full canonicalized path to the clipboard |
| Picker mouse click | Left-clicking a file-picker entry selects it and copies its full path to the clipboard |

Nothing here is upstreamed yet. If any of it is useful to the original project, it should go
there — the intent of this fork is personal use, not divergence.

Licensed MIT, the same as the original.

## Screenshots

| | |
|---|---|
| ![Demo](screenshots/demo.png) | ![Light Theme](screenshots/light.png) |
| ![Math Rendering](screenshots/math.png) | ![Mermaid Diagrams](screenshots/mermaid.png) |
| ![Search](screenshots/search.png) | ![File Picker](screenshots/file-picker.png) |

## Features

- **Interactive TUI** — Scroll, navigate with keyboard and mouse
- **Syntax highlighting** — Code blocks highlighted via syntect (base16-ocean.dark / InspiredGitHub themes)
- **Rich formatting** — Headings, bold, italic, strikethrough, lists, blockquotes, tables, interactive checkboxes
- **Inline images** — Renders images in the terminal via Kitty, iTerm2, or Unicode half-block fallback
- **Clickable links** — OSC 8 hyperlinks in supporting terminals
- **In-document search** — `/` to search with regex support, `n`/`N` to jump between matches
- **Table of contents** — Press `o` to browse and jump to any heading
- **File picker** — Start in a directory, search all nested `.md` files, and reopen it with `p`
- **Fuzzy heading search** — Press `:` to filter headings by name
- **Heading jumps** — `[` / `]` to jump between sections
- **Local file links** — Click or select relative markdown links to navigate between files, with `Backspace` to go back
- **Link picker** — Press `f` to list all links, type a number to open in browser
- **Click-to-copy** — Click any heading section, list, or code block to copy it; `Y` copies full document, `c` copies nearest code block, `Shift+P` copies the current file's full path
- **Wikilinks** — Obsidian-style `[[note]]` links and `![[embed]]` images/attachments render and navigate like ordinary Markdown
- **Quick Look preview** — `Enter` or click a wikilink embed's caption to open the real file at full quality in the OS's native previewer
- **Mermaid diagrams** — Visual rendering of flowcharts and sequence diagrams in the terminal with box-drawing characters
- **Math rendering** — LaTeX to Unicode: `$\alpha + \beta$` renders as `α + β`
- **Slide mode** — `--slides` treats `---` as slide separators for terminal presentations
- **Auto-reload** — Automatically detects file changes and reloads (via inotify/FSEvents/kqueue)
- **Stdin support** — Pipe markdown from any command: `curl ... | mdviewer`
- **Multiple files** — `mdviewer a.md b.md`, switch with `Tab` / `Shift+Tab`
- **HTML export** — `--export html` outputs themed, self-contained HTML
- **Dark/light themes** — Toggle with `t`, or set via `--theme` / config file
- **Line numbers** — Toggle with `l` for code blocks
- **Config file** — `~/.config/mdviewer/config.toml` for persistent preferences
- **Word wrapping** — Responsive re-wrapping on terminal resize
- **JSON viewer** — Render JSON files with syntax-colored keys, values, and structure
- **Pipe-friendly** — Outputs plain styled text when stdout is piped

## Installation

Requires Rust 1.85+ (edition 2024).

```bash
cargo install --path .
```

## Usage

```bash
mdviewer                              # pick a Markdown file from the current directory
mdviewer README.md                    # view a file
mdviewer docs/                        # pick a Markdown file from a directory
mdviewer a.md b.md                    # multiple files (Tab to switch)
mdviewer data.json                    # view a JSON file
cat README.md | mdviewer              # read from stdin
mdviewer --slides deck.md             # slide mode
mdviewer --export html doc.md > out.html  # export to HTML
mdviewer --theme light README.md      # light theme
mdviewer -l README.md                 # line numbers in code blocks
```

When piped, mdviewer outputs styled text without the interactive viewer:

```bash
mdviewer README.md | less -R
```

## Controls

### Navigation

| Key | Action |
|-----|--------|
| `j` / `Down` | Scroll down one line |
| `k` / `Up` | Scroll up one line |
| `Space` / `Page Down` | Page down |
| `b` / `Page Up` | Page up |
| `d` / `u` (or `Ctrl+d` / `Ctrl+u`) | Half-page down / up |
| `g` / `Home` | Jump to top |
| `G` / `End` | Jump to bottom |
| `[` / `]` | Previous / next heading |
| `Backspace` | Go back (after following a local file link) |
| Mouse scroll | Scroll up/down |

### Search

| Key | Action |
|-----|--------|
| `/` | Open search (supports regex) |
| `Enter` | Execute search |
| `n` / `N` | Next / previous match |
| `Esc` | Clear search |

### Features

| Key | Action |
|-----|--------|
| `o` | Table of contents overlay |
| `p` | File picker |
| `:` | Fuzzy heading search |
| `f` | Link picker (open URLs / follow local links) |
| `t` | Toggle dark/light theme |
| `l` | Toggle line numbers in code blocks |
| Click heading | Copy heading section to clipboard |
| Click list | Copy entire list to clipboard |
| Click code block | Copy code block to clipboard |
| `Y` | Copy entire document to clipboard |
| `c` | Copy nearest code block to clipboard |
| `Shift+P` | Copy the current file's full path to clipboard |
| `Tab` / `Shift+Tab` | Switch between files |
| `h` / `?` / `F1` | Help screen |
| `q` / `Ctrl+C` | Quit |

### File Picker

When launched without file arguments, mdviewer opens a file picker rooted at the current directory. Passing a directory path opens the picker at that directory. Pressing `p` while viewing a file also roots the picker at the directory mdviewer was launched from — not the open file's folder — so it searches the same place your shell would. Type to search across all nested `.md` paths; the query is matched as a fuzzy subsequence, so a path like `hello/world/a.md` can be found with `hellrlda.md`.

| Key | Action |
|-----|--------|
| Type text | Search Markdown files |
| `Up` / `Down` | Select previous / next file |
| `Page Up` / `Page Down` | Move by page |
| `Home` / `End` | Jump to first / last result |
| `Enter` | Open selected file |
| Click entry | Select the file and copy its full path to clipboard |
| `F5` | Refresh file list |
| `Esc` | Close picker after a file is open |

By default results are sorted alphabetically. Set `[picker] sort_by = "mtime,desc"` to
show the most recently edited files first instead — see
[Sorting the picker's results](#sorting-the-pickers-results).

### Slide Mode (`--slides`)

| Key | Action |
|-----|--------|
| `Right` / `Space` / `l` / `j` / `Down` / `Page Down` | Next slide |
| `Left` / `b` / `h` / `k` / `Up` / `Page Up` | Previous slide |
| `g` / `Home` | First slide |
| `G` / `End` | Last slide |

## Configuration

### Where the config file goes

Put it at **`~/.config/mdviewer/config.toml`** — on every platform, including macOS.

The lookup order is:

| Order | Location |
|-------|----------|
| 1 | `$XDG_CONFIG_HOME/mdviewer/config.toml` (only if `XDG_CONFIG_HOME` is set) |
| 2 | `~/.config/mdviewer/config.toml` |
| 3 | Platform config dir — on macOS `~/Library/Application Support/mdviewer/config.toml`, on Linux `~/.config/`, on Windows `%APPDATA%` |

The first file that **exists** wins; the rest are ignored. Nothing is created for you.

An `mdterm/` directory is still accepted at each of those locations, so a config written
before this fork was renamed keeps working.

```bash
mkdir -p ~/.config/mdviewer
$EDITOR ~/.config/mdviewer/config.toml
```

### A complete example

Every key is optional — omit anything you don't care about.

```toml
theme = "dark"        # "dark" or "light"
line_numbers = false  # line numbers inside code blocks
width = 0             # display width, 0 = auto-detect
# attachment_folder_path = "attachments"  # extra subfolder checked when resolving ![[bare-filename]] embeds

# Content that is parsed but never drawn.
[hide]
images = true                              # skip images: no placeholder, no caption, no download
images_in_links = true                     # skip only images nested inside a link's label
frontmatter = true                         # skip a leading `---` YAML block
code_languages = ["dataviewjs", "dataview"]  # fenced languages to drop entirely

# Which files the file picker offers.
[picker]
ignore = ["attachments", "Templates"]  # skip these file/directory names
hidden = false                         # false = skip dot-dirs (.obsidian, .git, .trash)
max_results = 0                        # cap the list length; 0 = limited only by terminal height
# sort_by = "mtime,desc"               # sort by last-modified time instead of alphabetically
```

### Verifying it is being read

Config only affects rendering, so compare with and without it:

```bash
mdviewer --no-color note.md | grep -c dataviewjs   # 0 once the config is in place
```

### Hiding content

`[hide]` drops content at render time. It is never drawn, and hidden images are never even
fetched from the network. Hidden code blocks are also excluded from `c` (copy nearest code
block), so they can't be copied by accident.

- `images` — drops the image placeholder rows *and* the caption line
- `images_in_links` — drops only an image nested inside a link's label (e.g. a decorative
  icon prefixing a link), leaving the link's other text intact. If the image *is* the
  entire link label (no other text), the whole link disappears — nothing is left to click
- `frontmatter` — drops a leading `---` … `---` block. A document that merely opens with a
  `---` rule and never closes it is left alone
- `code_languages` — matched against the first word of the fence info string, so
  ```` ```dataviewjs foo=bar ```` still matches `dataviewjs`. Case-insensitive

Useful for Obsidian vaults, where notes carry frontmatter and ```` ```dataviewjs ```` blocks
that mean nothing outside Obsidian:

```bash
mdviewer --no-images --no-frontmatter --hide-code-lang dataviewjs note.md
```

### Ignoring files in the picker

`[picker] ignore` entries match a **single path component** — a file or directory name, not
a path or a glob. `"attachments"` skips that directory anywhere in the tree; `"notes.md"`
skips any file with that name. Matching is case-insensitive.

Dot-directories are skipped by default, the same as `fd`, which keeps `.obsidian/`, `.git/`
and `.trash/` out of the picker. Set `hidden = true` to include them.

### Sorting the picker's results

By default, results are sorted alphabetically. `[picker] sort_by` switches to sorting by
last-modified time instead:

```toml
[picker]
sort_by = "mtime,desc"  # newest first; "mtime,asc" for oldest first
```

Sorting applies underneath the fuzzy match ranking: results are still ordered by match
quality first when you've typed a query, with `sort_by` only deciding the order among
equally-good matches (and the order of the full, unfiltered list when the query is empty).
Any value other than `"mtime,desc"` / `"mtime,asc"` falls back to alphabetical.

### Limiting how many results are listed

`[picker] max_results` caps the length of the list, so opening the picker on a large vault
doesn't fill the screen:

```toml
[picker]
max_results = 5
```

This is **a display cap only**. Every file is still matched against your query, so anything
in the tree can be found by typing — you just see at most 5 rows at a time, and can scroll
through the rest. The counter in the box header keeps showing the true totals, so
`237/1041` means 237 files match your query out of 1041 discovered.

`0` (the default) means the list is limited only by terminal height.

### Finding attachments outside the ancestor search

A wikilink embed by bare filename, `![[photo.png]]`, resolves by checking the note's own
directory, then walking upward through its ancestors until a `photo.png` turns up. That
covers a vault with one shared attachments folder sitting *above* every note.

It does **not** cover a folder that sits *alongside* notes instead — e.g. every note
directory has its own nested `attachments/` next to it, rather than one shared folder
higher up the tree. `attachment_folder_path` adds that as one more candidate at each level
of the same search: alongside `<dir>/photo.png`, it also tries
`<dir>/<attachment_folder_path>/photo.png` before moving up to the next ancestor.

```toml
attachment_folder_path = "attachments"
```

Applies to wikilink embeds only (`![[...]]`) — plain CommonMark images (`![](path)`) are
unaffected, same as the ancestor search itself.

### CLI vs config

CLI flags override config file settings, with two exceptions that **add** to the config
rather than replacing it: `--hide-code-lang` appends to `code_languages`, and the `--no-*`
flags can only turn hiding on, never off.

## CLI Reference

```
mdviewer [OPTIONS] [FILES]...

Arguments:
  [FILES]...               Markdown file(s) to view, or a directory to pick from

Options:
  -T, --theme <THEME>      Theme: dark or light
  -w, --width <WIDTH>      Display width override (0 = auto)
  -s, --slides             Slide mode (--- as slide separators)
  -l, --line-numbers       Show line numbers in code blocks
      --export <FORMAT>    Export format (html)
      --no-color           Disable colors
      --no-images          Skip images entirely instead of rendering them
      --no-frontmatter     Skip a leading YAML frontmatter block
      --hide-code-lang <LANG>
                           Omit fenced code blocks with this language (repeatable)
  -h, --help               Print help
  -V, --version            Print version
```

## Building

```bash
cargo build --release
```

## Demo

![Demo](demo.gif)

## License

MIT
