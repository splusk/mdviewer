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
| `[picker] external_command` | Runs this via `$SHELL -i -c` instead of the built-in picker, e.g. to delegate to a personal `fzf`-based shell function |
| Picker rooting | Pressing `p` roots the picker at the directory mdviewer was launched from, rather than the open file's folder |
| Picker key fix | `p` and `q` are literal search characters in the picker; previously they closed it — or quit outright — so no query containing them could be typed |
| Picker query reset | The picker's search text no longer persists across close/reopen — previously it kept whatever was last typed |
| Config lookup | Searches `$XDG_CONFIG_HOME`, then `~/.config`, then the platform directory, so the documented `~/.config/…/config.toml` works on macOS too (see [mdterm#66](https://github.com/bahdotsh/mdterm/issues/66)) |

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
- **Click-to-copy** — Click any heading section, list, or code block to copy it; `Y` copies full document, `c` copies nearest code block
- **Mermaid diagrams** — Visual rendering of flowcharts/graphs in the terminal with box-drawing characters
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
| `F5` | Refresh file list |
| `Esc` | Close picker after a file is open |

Set `[picker] external_command` to shell out to something else entirely instead — see
[Using an external picker command](#using-an-external-picker-command).

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
# external_command = "p"               # if set, `p` runs this instead of the built-in picker
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

### Using an external picker command

The built-in picker is a self-contained fuzzy finder — it doesn't read `.gitignore`/`.ignore`
files, doesn't sort by modification time, and knows nothing about your shell's `fzf` setup.
If you already have a `fzf`-based file picker wired up in your shell (e.g. a `p()` function
bound to a key), `[picker] external_command` delegates to it instead:

```toml
[picker]
external_command = "p"
```

Pressing `p` then suspends mdviewer's TUI, runs the command via `$SHELL -i -c "<command>"` with
its working directory set to the picker's root (so it searches the same place your shell would),
and resumes once it exits. The `-i` flag means an interactive shell runs it, so shell functions
and rc-file-defined aliases resolve — not just binaries on `PATH`. mdviewer doesn't inspect the
command's output or exit status; whatever it does (open a file, open a nested mdviewer, cancel)
is between you and your shell, exactly as if you'd run it from a plain terminal.

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
