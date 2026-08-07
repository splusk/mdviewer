# Design: Mermaid sequence diagram rendering

**Date:** 2026-08-07
**Branch:** `feat/sequence-diagrams` (off `main`)

## Problem

`diagram.rs` only understands Mermaid **flowchart** syntax (`graph`/`flowchart`).
Its parser has no concept of `sequenceDiagram`. When a `sequenceDiagram` code
block is rendered, every line (`participant X as ...`, `A->>B: ...`, `loop
...`, `Note over ...`) falls through to the generic flowchart line-parser,
which extracts stray words as bogus "nodes" and arrows as edges. The result is
a scrambled box-per-word layout with no relation to the actual diagram (see
attached screenshots: boxes labeled `SvcA-`, `Backend--`, `loop`, `Note`,
`sequenceDiagram`, etc.)

Root cause: this is a missing feature, not a parser bug — the flowchart parser
was never meant to handle sequence-diagram syntax, and nothing detects the
diagram type before feeding it to that parser.

## Scope

In scope:
- `participant`/`actor` declarations (`participant X` / `participant X as
  Label`), plus auto-declaring participants in first-appearance order if used
  without a prior `participant` line (matches real Mermaid).
- Message arrows: `->>`, `-->>` (arrowhead), `->`, `-->` (no arrowhead), `-x`,
  `--x` (lost-message cross terminator). `--` = dashed line, single `-` =
  solid line.
- Activation: `+`/`-` suffix on the target/source of an arrow, and standalone
  `activate X` / `deactivate X`.
- `Note over A[,B]`, `Note left of A`, `Note right of A` (single-line text).
- `loop`, `alt`, `opt`, `par` blocks, closed by `end`, with `else` (alt) and
  `and` (par) section dividers. Nesting is supported (stack-based).
- Self-messages (`A->>A: ...`).
- Comments (`%%`) and unrecognized directives (`autonumber`, `title`, etc.)
  are explicitly skipped — never fall through to generic line parsing.

Out of scope (explicitly, per user decision):
- `par` branches rendered as side-by-side lanes — rendered instead as
  sequential labeled sections, same as `alt`/`else`.
- `rect`/background coloring, `autonumber`, multi-line notes, box grouping
  (`box ... end`).
- Anything not listed above falls back to the existing plain syntax-highlighted
  code block (i.e. `parse_sequence` returns `None`), identical to how an
  unparseable flowchart already behaves.

## Module restructure

`diagram.rs` (1135 lines) currently mixes shared ASCII-canvas primitives with
flowchart-only parsing/layout/rendering. Splitting it into a directory module
keeps each file focused on one diagram type, and keeps the new sequence-diagram
code (parser + layout + renderer, comparable in size to the existing flowchart
code) from making a single file unmanageably large:

```
src/diagram/
  mod.rs        - Canvas, CanvasCell, NodeShape, CONN_* / junction_char,
                  CardDrawRow / draw_card (used by json.rs), and the public
                  render_mermaid() dispatcher.
  flowchart.rs  - today's Graph, parse_mermaid, parse_line, parse_arrow,
                  parse_node_ref, assign_layers, order_within_layers,
                  node_box_width, render_td, render_lr. Moved verbatim aside
                  from `pub(crate)` visibility adjustments needed to be
                  reached from mod.rs.
  sequence.rs   - new: SequenceDiagram model, parse_sequence, layout, render.
```

`json.rs` imports `crate::diagram::{Canvas, CardDrawRow}` today — both stay
defined in `mod.rs`, so that import is unaffected by the split.

### Dispatcher

```rust
// mod.rs
pub fn render_mermaid(code: &str, theme: &Theme) -> Option<(Vec<Vec<StyledSpan>>, usize)> {
    let first_line = code
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty() && !l.starts_with("%%"));

    if first_line == Some("sequenceDiagram") {
        return sequence::render_sequence(code, theme);
    }
    flowchart::render_flowchart(code, theme)
}
```

This is the fix for the actual bug: sequence-diagram source now never reaches
the flowchart parser at all.

## Sequence diagram data model (`sequence.rs`)

```rust
struct Participant { id: String, label: String, center_x: usize, width: usize }

enum LineStyle { Solid, Dashed }
enum ArrowEnd { Arrowhead, None, Cross }

struct Message {
    from: String,
    to: String,
    text: Option<String>,
    line: LineStyle,
    end: ArrowEnd,
    activate: bool,    // '+' on this arrow
    deactivate: bool,  // '-' on this arrow
}

enum NoteTarget { Over(String, Option<String>), LeftOf(String), RightOf(String) }
struct Note { target: NoteTarget, text: String }

struct BlockSection { label: Option<String>, events: Vec<Event> } // label: "loop text" / "alt text" / "else text" / "and text" / "opt text" / "par text"
struct Block { keyword: &'static str, sections: Vec<BlockSection> }

enum Event {
    Message(Message),
    Note(Note),
    Activate(String),
    Deactivate(String),
    Block(Block),
}

struct SequenceDiagram {
    participants: Vec<Participant>, // declaration/first-appearance order
    events: Vec<Event>,
}
```

### Parsing

`parse_sequence(code: &str) -> Option<SequenceDiagram>`:

1. Require the first non-blank/non-`%%` line to be exactly `sequenceDiagram`
   (already guaranteed by the dispatcher, but `parse_sequence` re-checks so it
   stays a valid standalone function/unit-testable in isolation).
2. Walk remaining lines with a mutable block stack (`Vec<Block>` being built).
   A `loop|alt|opt|par <text>` line pushes a new `Block` with one open
   `BlockSection`; `else <text>` / `and <text>` closes the current section and
   opens a new one within the same `Block`; `end` pops the completed `Block`
   and appends it as an `Event` to whichever level is now on top (or to the
   top-level `events` if the stack is empty).
3. `participant X [as Label]` / `actor X [as Label]` registers a
   `Participant` (label defaults to `X`) if not already present, in
   declaration order.
4. A message line is recognized by matching one of the arrow tokens
   (longest-first: `-->>`, `->>`, `--x`, `-x`, `-->`, `->`) between two
   identifiers, splitting `<from><arrow><to>: <text>`. `+`/`-` immediately
   before the target identifier sets `activate`/`deactivate`. Any
   participant referenced here that wasn't declared is auto-registered at
   first appearance, per Mermaid's own behavior.
5. `Note over A[,B]: text`, `Note left of A: text`, `Note right of A: text`
   parsed into `Event::Note`.
6. `activate X` / `deactivate X` parsed into their own events.
7. Anything else (blank, `%%...`, `autonumber`, `title ...`, unrecognized) is
   skipped — this is the safety net that keeps unsupported syntax from ever
   reaching a "parse words as nodes" fallback.
8. Returns `None` if no participants were ever registered (mirrors
   flowchart's `nodes.is_empty()` bail-out) — falls back to the plain code
   block, same UX as an empty/garbled flowchart today.

### Layout

Sequence diagrams are linear in time, so this is a single top-to-bottom pass —
simpler than the flowchart's topological-sort + barycenter-reduction layout:

1. **Columns.** Each participant gets a box width (`label.len() + 4`, min 10,
   matching `flowchart::label_box_width`'s spirit) and a `center_x`, laid out
   left-to-right in declaration order with a fixed gap (reuses the flowchart
   module's existing gap constants for visual consistency).
2. **Rows.** Walk `events` recursively (blocks recurse into their sections),
   accumulating a `y` cursor and recording a `(y_start, y_end, x_start,
   x_end)` span per event:
   - `Message`: 2 rows (label row above, arrow row below). Self-message
     (`from == to`): 3 rows (a small right-side bump: line out, label, line
     back with arrowhead/cross).
   - `Note`: 3 rows (top border, single text row, bottom border). Box width =
     `text.len() + 4`, min 10; horizontal position per `NoteTarget` (`Over`
     centers on the midpoint of the two participants' `center_x`; `LeftOf`/
     `RightOf` sit just outside the target's column with a small gap).
   - `Activate`/`Deactivate` (standalone statements): 0 rows — they only
     toggle the "active" flag used by lifeline rendering, at the current `y`.
   - `Block` section boundary (top of block, each `else`/`and` divider,
     bottom of block): 1 row each. A block's own `x_start`/`x_end` is the
     min/max over every event nested inside it (recursively), expanded by a
     small margin, so `loop`/`alt`/etc. borders visually enclose exactly the
     participants they involve.
3. **Canvas size.** Width = rightmost participant/box/note edge across the
   whole diagram + margin. Height = final `y` cursor + margin.

### Rendering

Reuses `mod.rs`'s `Canvas`/`draw_node`-style primitives rather than
introducing a second drawing abstraction:

1. Draw participant label boxes (`Canvas::draw_node`, `NodeShape::Rectangle`)
   at the top.
2. Draw lifelines (`│`) for every participant, for the full canvas height,
   *before* anything else — later draws intentionally overwrite lifeline
   cells they cross (same "later writes win" pattern `Canvas` already uses
   for flowchart edges over background cells).
3. Walk events in the same order as layout, drawing each into its assigned
   rows:
   - Message: label centered on the label row; arrow row fills between
     `from.center_x` and `to.center_x` with `─` (solid) or `┄` (dashed),
     terminated by `▶`/`◀` (direction-dependent) or `✗` (cross). Lifelines at
     every *other* participant column continue as `│`/`┃` through this row
     (only the src/dst segment is overwritten).
   - Self-message: small bump to the right of the lifeline, as scoped above.
   - Note: box border/text via the same node-box drawing helper used for
     participants, just without the "participant row" semantics.
   - Block boundary: horizontal border from `x_start` to `x_end` at the
     assigned row; where that row crosses a *currently-live* lifeline column,
     draw `┬` (top border) / `┴` (bottom border) / `┼`-if-both instead of a
     plain `─`, so the block visually reads as enclosing the lifelines rather
     than cutting them. Corners stay `┌┐└┘`/`├┤` (no extra junction logic for
     the corner case — acceptable, minor visual simplification). The keyword
     + label (e.g. `loop Every request`, `else other`) is drawn left-aligned
     just inside the top-left corner of its section, matching Mermaid's own
     visual convention.
4. **Activation bars.** Track a `HashSet<participant_id>` of currently-active
   participants while walking events in order; while a participant is active,
   its lifeline character is `┃` (heavy vertical) instead of `│`, colored with
   `theme.h3` (the same "distinct from border" color already used for edge
   labels in the flowchart renderer) to visually set it apart.

## Testing plan

Following the existing `#[cfg(test)]` module-in-file pattern (`style.rs`,
`json.rs`, `wikilink.rs`, etc.):

- `sequence.rs` parser unit tests: participant declaration (explicit +
  auto-registered), each arrow/line-style combination, activation via `+`/`-`
  and standalone `activate`/`deactivate`, each `Note` variant, nested
  `loop`/`alt`/`else` blocks, self-message, comment/unrecognized-directive
  lines are skipped without producing bogus participants.
- `sequence.rs` layout unit tests: column `center_x` ordering matches
  declaration order; a block's computed `x_start`/`x_end` matches the min/max
  of its nested participants; row-height accumulation for a small fixed
  sequence matches hand-computed expected `y` values.
- `mod.rs` dispatcher unit test: a code block starting with `sequenceDiagram`
  routes to `sequence::render_sequence` (e.g. assert a known sequence-only
  construct like `Note over` doesn't panic/garble when a flowchart-only line
  is absent), and a `graph TD` block still routes to `flowchart::render_flowchart`
  (regression test for the split not breaking existing flowchart rendering).
- Rendered-output test using the screenshot's actual example (5 participants,
  a `loop`, a `Note over`, activation-free) as an integration-style snapshot:
  assert specific expected substrings/positions in the output rows (e.g. each
  participant label appears in its own column, `loop Every request` appears
  once, no stray single-word boxes).
- Full existing flowchart test suite (once added/verified during the module
  split) must still pass unchanged — the split moves code, it must not change
  flowchart behavior.
- Verification: no local Rust toolchain in this environment; run
  `docker run --rm -v "$PWD":/w -w /w rust:1-slim cargo test` (clearing
  `target/` first if a stale-incremental error occurs, per prior sessions in
  this repo) before considering the work done.
