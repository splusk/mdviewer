use crossterm::style::Color;

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Participant {
    pub(crate) id: String,
    pub(crate) label: String,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum LineStyle {
    Solid,
    Dashed,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum ArrowEnd {
    Arrowhead,
    None,
    Cross,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Message {
    pub(crate) from: String,
    pub(crate) to: String,
    pub(crate) text: Option<String>,
    pub(crate) line: LineStyle,
    pub(crate) end: ArrowEnd,
    pub(crate) activate: bool,
    pub(crate) deactivate: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum NoteTarget {
    Over(String, Option<String>),
    LeftOf(String),
    RightOf(String),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Note {
    pub(crate) target: NoteTarget,
    pub(crate) text: String,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct BlockSection {
    pub(crate) label: String,
    pub(crate) events: Vec<Event>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Block {
    pub(crate) sections: Vec<BlockSection>,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Event {
    Message(Message),
    Note(Note),
    Activate(String),
    Deactivate(String),
    Block(Block),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct SequenceDiagram {
    pub(crate) participants: Vec<Participant>,
    pub(crate) events: Vec<Event>,
}

pub(crate) fn parse_sequence(code: &str) -> Option<SequenceDiagram> {
    let mut lines = code.lines().map(str::trim);
    let header = lines.find(|l| !l.is_empty() && !l.starts_with("%%"));
    if header != Some("sequenceDiagram") {
        return None;
    }

    let mut participants: Vec<Participant> = Vec::new();
    let mut block_stack: Vec<Block> = Vec::new();
    let mut event_stack: Vec<Vec<Event>> = vec![Vec::new()];

    for line in lines {
        if line.is_empty() || line.starts_with("%%") {
            continue;
        }

        if let Some(rest) = line.strip_prefix("participant ") {
            register_participant(rest, &mut participants);
            continue;
        }
        if let Some(rest) = line.strip_prefix("actor ") {
            register_participant(rest, &mut participants);
            continue;
        }
        if let Some(rest) = line.strip_prefix("activate ") {
            event_stack.last_mut().unwrap().push(Event::Activate(rest.trim().to_string()));
            continue;
        }
        if let Some(rest) = line.strip_prefix("deactivate ") {
            event_stack.last_mut().unwrap().push(Event::Deactivate(rest.trim().to_string()));
            continue;
        }
        if let Some(rest) = line.strip_prefix("Note ") {
            if let Some(note) = parse_note(rest) {
                for id in note_participant_ids(&note.target) {
                    register_participant(&id, &mut participants);
                }
                event_stack.last_mut().unwrap().push(Event::Note(note));
            }
            continue;
        }

        if let Some(rest) = keyword_rest(line, "loop")
            .or_else(|| keyword_rest(line, "alt"))
            .or_else(|| keyword_rest(line, "opt"))
            .or_else(|| keyword_rest(line, "par"))
        {
            let keyword = line.split_whitespace().next().unwrap_or(line);
            let label = if rest.is_empty() {
                keyword.to_string()
            } else {
                format!("{keyword} {rest}")
            };
            block_stack.push(Block {
                sections: vec![BlockSection { label, events: Vec::new() }],
            });
            event_stack.push(Vec::new());
            continue;
        }

        if let Some(rest) = keyword_rest(line, "else").or_else(|| keyword_rest(line, "and")) {
            if let Some(block) = block_stack.last_mut() {
                let keyword = if line.starts_with("else") { "else" } else { "and" };
                let label = if rest.is_empty() {
                    keyword.to_string()
                } else {
                    format!("{keyword} {rest}")
                };
                let closed_events = event_stack.pop().unwrap();
                block.sections.last_mut().unwrap().events = closed_events;
                block.sections.push(BlockSection { label, events: Vec::new() });
                event_stack.push(Vec::new());
            }
            continue;
        }

        if line == "end" {
            if let Some(mut block) = block_stack.pop() {
                let closed_events = event_stack.pop().unwrap();
                block.sections.last_mut().unwrap().events = closed_events;
                event_stack.last_mut().unwrap().push(Event::Block(block));
            }
            continue;
        }

        if let Some((spec, text)) = split_message_text(line)
            && let Some((from_id, style, end, activate, to_id, deactivate)) = split_message(spec)
        {
            register_participant(&from_id, &mut participants);
            register_participant(&to_id, &mut participants);
            event_stack.last_mut().unwrap().push(Event::Message(Message {
                from: from_id,
                to: to_id,
                text,
                line: style,
                end,
                activate,
                deactivate,
            }));
        }
        // Anything else (blank/malformed/unrecognized directives like
        // `autonumber`/`title`) is silently skipped — never fall through to
        // generic parsing, which is what caused the original bug.
    }

    while let Some(mut block) = block_stack.pop() {
        let closed_events = event_stack.pop().unwrap();
        block.sections.last_mut().unwrap().events = closed_events;
        event_stack.last_mut().unwrap().push(Event::Block(block));
    }

    if participants.is_empty() {
        return None;
    }

    Some(SequenceDiagram {
        participants,
        events: event_stack.pop().unwrap(),
    })
}

/// Matches `line == keyword` (returns `Some("")`) or `line == "<keyword> <rest>"`
/// (returns `Some(rest)`), so callers can handle both a bare block keyword and
/// one followed by a condition/label in one call.
fn keyword_rest<'a>(line: &'a str, keyword: &str) -> Option<&'a str> {
    if line == keyword {
        Some("")
    } else {
        line.strip_prefix(keyword).and_then(|r| r.strip_prefix(' '))
    }
}

fn register_participant(spec: &str, participants: &mut Vec<Participant>) {
    let spec = spec.trim();
    let (id, label) = match spec.split_once(" as ") {
        Some((id, label)) => (id.trim().to_string(), label.trim().to_string()),
        None => (spec.to_string(), spec.to_string()),
    };
    if id.is_empty() {
        return;
    }
    if !participants.iter().any(|p| p.id == id) {
        participants.push(Participant { id, label });
    }
}

fn parse_note(rest: &str) -> Option<Note> {
    let (target, text) = rest.split_once(':')?;
    let target = target.trim();
    let text = text.trim().to_string();

    let note_target = if let Some(names) = target.strip_prefix("over ") {
        let mut parts = names.split(',');
        let a = parts.next()?.trim().to_string();
        let b = parts.next().map(|s| s.trim().to_string());
        NoteTarget::Over(a, b)
    } else if let Some(name) = target.strip_prefix("left of ") {
        NoteTarget::LeftOf(name.trim().to_string())
    } else if let Some(name) = target.strip_prefix("right of ") {
        NoteTarget::RightOf(name.trim().to_string())
    } else {
        return None;
    };

    Some(Note { target: note_target, text })
}

pub(crate) fn note_participant_ids(target: &NoteTarget) -> Vec<String> {
    match target {
        NoteTarget::Over(a, Some(b)) => vec![a.clone(), b.clone()],
        NoteTarget::Over(a, None) => vec![a.clone()],
        NoteTarget::LeftOf(a) | NoteTarget::RightOf(a) => vec![a.clone()],
    }
}

/// Splits `"<spec>: <text>"` into `(spec, Some(text))`, or `("<spec>", None)`
/// if there's no `:`. Only the *first* `:` is a delimiter — message text may
/// itself contain colons.
fn split_message_text(line: &str) -> Option<(&str, Option<String>)> {
    match line.split_once(':') {
        Some((spec, text)) => {
            let text = text.trim();
            Some((spec.trim(), if text.is_empty() { None } else { Some(text.to_string()) }))
        }
        None => Some((line.trim(), None)),
    }
}

/// Splits `"<from><arrow>[+|-]<to>"` into its parts. Returns `None` if no
/// arrow token is found (the line isn't a message — safe to skip).
fn split_message(spec: &str) -> Option<(String, LineStyle, ArrowEnd, bool, String, bool)> {
    const TOKENS: [(&str, LineStyle, ArrowEnd); 6] = [
        ("-->>", LineStyle::Dashed, ArrowEnd::Arrowhead),
        ("-->", LineStyle::Dashed, ArrowEnd::None),
        ("->>", LineStyle::Solid, ArrowEnd::Arrowhead),
        ("->", LineStyle::Solid, ArrowEnd::None),
        ("--x", LineStyle::Dashed, ArrowEnd::Cross),
        ("-x", LineStyle::Solid, ArrowEnd::Cross),
    ];

    let mut best: Option<(usize, &str, LineStyle, ArrowEnd)> = None;
    for (token, style, end) in TOKENS {
        if let Some(idx) = spec.find(token) {
            let replace = match best {
                None => true,
                Some((best_idx, ..)) => idx < best_idx,
            };
            if replace {
                best = Some((idx, token, style, end));
            }
        }
    }
    let (idx, token, style, end) = best?;

    let from = spec[..idx].trim().to_string();
    if from.is_empty() {
        return None;
    }

    let rest = spec[idx + token.len()..].trim_start();
    let (activate, rest) = match rest.strip_prefix('+') {
        Some(r) => (true, r),
        None => (false, rest),
    };
    let (deactivate, rest) = match rest.strip_prefix('-') {
        Some(r) => (true, r),
        None => (false, rest),
    };

    let to = rest.trim().to_string();
    if to.is_empty() {
        return None;
    }

    Some((from, style, end, activate, to, deactivate))
}

pub(crate) struct Column {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) center_x: usize,
    pub(crate) width: usize,
}

pub(crate) enum Positioned {
    Message {
        from_x: usize,
        to_x: usize,
        label_y: usize,
        arrow_y: usize,
        text: Option<String>,
        line: LineStyle,
        end: ArrowEnd,
    },
    SelfMessage {
        x: usize,
        top_y: usize,
        label_y: usize,
        bottom_y: usize,
        text: Option<String>,
        end: ArrowEnd,
    },
    Note {
        x_start: usize,
        x_end: usize,
        top_y: usize,
        text: String,
    },
    Border {
        x_start: usize,
        x_end: usize,
        y: usize,
        label: Option<String>,
        top: bool,
    },
}

pub(crate) struct Layout {
    pub(crate) columns: Vec<Column>,
    pub(crate) positioned: Vec<Positioned>,
    pub(crate) active_spans: Vec<(String, usize, usize)>,
    pub(crate) width: usize,
    pub(crate) height: usize,
}

const COLUMN_GAP: usize = 4;
const LEFT_MARGIN: usize = 2;
const BLOCK_MARGIN: usize = 3;

fn box_width(label: &str) -> usize {
    (label.chars().count() + 4).max(10)
}

fn build_columns(participants: &[Participant]) -> Vec<Column> {
    let mut x = LEFT_MARGIN;
    participants
        .iter()
        .map(|p| {
            let width = box_width(&p.label);
            let center_x = x + width / 2;
            x += width + COLUMN_GAP;
            Column { id: p.id.clone(), label: p.label.clone(), center_x, width }
        })
        .collect()
}

fn column_center(columns: &[Column], id: &str) -> Option<usize> {
    columns.iter().find(|c| c.id == id).map(|c| c.center_x)
}

fn note_span(columns: &[Column], target: &NoteTarget) -> (usize, usize) {
    match target {
        NoteTarget::Over(a, Some(b)) => {
            let xa = column_center(columns, a).unwrap_or(0);
            let xb = column_center(columns, b).unwrap_or(0);
            (xa.min(xb), xa.max(xb))
        }
        NoteTarget::Over(a, None) => {
            let x = column_center(columns, a).unwrap_or(0);
            (x, x)
        }
        NoteTarget::LeftOf(a) => {
            let x = column_center(columns, a).unwrap_or(0);
            (x.saturating_sub(4), x.saturating_sub(2))
        }
        NoteTarget::RightOf(a) => {
            let x = column_center(columns, a).unwrap_or(0);
            (x + 2, x + 4)
        }
    }
}

fn center_span(mid_start: usize, mid_end: usize, box_w: usize) -> (usize, usize) {
    let mid = (mid_start + mid_end) / 2;
    let half = box_w / 2;
    let start = mid.saturating_sub(half);
    (start, start + box_w)
}

fn widen(columns: &[Column], id: &str, min_x: &mut usize, max_x: &mut usize) {
    if let Some(x) = column_center(columns, id) {
        *min_x = (*min_x).min(x);
        *max_x = (*max_x).max(x);
    }
}

fn scan_events_span(events: &[Event], columns: &[Column], min_x: &mut usize, max_x: &mut usize) {
    for event in events {
        match event {
            Event::Message(m) => {
                widen(columns, &m.from, min_x, max_x);
                widen(columns, &m.to, min_x, max_x);
            }
            Event::Note(n) => {
                for id in note_participant_ids(&n.target) {
                    widen(columns, &id, min_x, max_x);
                }
            }
            Event::Activate(id) | Event::Deactivate(id) => widen(columns, id, min_x, max_x),
            Event::Block(b) => {
                for section in &b.sections {
                    scan_events_span(&section.events, columns, min_x, max_x);
                }
            }
        }
    }
}

fn block_span(sections: &[BlockSection], columns: &[Column]) -> (usize, usize) {
    let mut min_x = usize::MAX;
    let mut max_x = 0usize;
    for section in sections {
        scan_events_span(&section.events, columns, &mut min_x, &mut max_x);
    }
    if min_x == usize::MAX {
        min_x = columns.first().map(|c| c.center_x).unwrap_or(0);
        max_x = columns.last().map(|c| c.center_x).unwrap_or(0);
    }
    (min_x.saturating_sub(BLOCK_MARGIN), max_x + BLOCK_MARGIN)
}

#[allow(clippy::too_many_arguments)]
fn layout_events(
    events: &[Event],
    columns: &[Column],
    cursor: &mut usize,
    positioned: &mut Vec<Positioned>,
    active_spans: &mut Vec<(String, usize, usize)>,
    active_since: &mut std::collections::HashMap<String, usize>,
    max_x: &mut usize,
) {
    for event in events {
        match event {
            Event::Message(m) => {
                let from_x = column_center(columns, &m.from).unwrap_or(0);
                let to_x = column_center(columns, &m.to).unwrap_or(0);

                if m.from == m.to {
                    let top_y = *cursor;
                    let label_y = top_y + 1;
                    let bottom_y = top_y + 2;
                    positioned.push(Positioned::SelfMessage {
                        x: from_x,
                        top_y,
                        label_y,
                        bottom_y,
                        text: m.text.clone(),
                        end: m.end,
                    });
                    *max_x = (*max_x).max(from_x + 8);
                    *cursor = bottom_y + 1;
                } else {
                    let label_y = *cursor;
                    let arrow_y = label_y + 1;
                    positioned.push(Positioned::Message {
                        from_x,
                        to_x,
                        label_y,
                        arrow_y,
                        text: m.text.clone(),
                        line: m.line,
                        end: m.end,
                    });
                    *max_x = (*max_x).max(from_x.max(to_x) + 2);
                    *cursor = arrow_y + 1;
                }

                // "+" activates the message's destination (the callee).
                // "-" deactivates the message's SOURCE, not its destination
                // — e.g. in `B-->>-A: reply`, it's B that deactivates, even
                // though `-` sits right before `A`. This matches Mermaid's
                // own activation shorthand semantics.
                if m.activate {
                    active_since.insert(m.to.clone(), *cursor);
                }
                if m.deactivate
                    && let Some(start) = active_since.remove(&m.from)
                {
                    active_spans.push((m.from.clone(), start, *cursor));
                }
            }
            Event::Activate(id) => {
                active_since.insert(id.clone(), *cursor);
            }
            Event::Deactivate(id) => {
                if let Some(start) = active_since.remove(id) {
                    active_spans.push((id.clone(), start, *cursor));
                }
            }
            Event::Note(n) => {
                let (mid_start, mid_end) = note_span(columns, &n.target);
                let min_box_w = box_width(&n.text);
                let box_w = min_box_w.max(mid_end.saturating_sub(mid_start) + 4);
                let (x_start, x_end) = center_span(mid_start, mid_end, box_w);
                let top_y = *cursor;
                positioned.push(Positioned::Note { x_start, x_end, top_y, text: n.text.clone() });
                *max_x = (*max_x).max(x_end + 2);
                *cursor = top_y + 3;
            }
            Event::Block(b) => {
                let (x_start, x_end) = block_span(&b.sections, columns);
                for (i, section) in b.sections.iter().enumerate() {
                    positioned.push(Positioned::Border {
                        x_start,
                        x_end,
                        y: *cursor,
                        label: Some(section.label.clone()),
                        top: true,
                    });
                    *cursor += 1;
                    layout_events(
                        &section.events,
                        columns,
                        cursor,
                        positioned,
                        active_spans,
                        active_since,
                        max_x,
                    );
                    let _ = i;
                }
                positioned.push(Positioned::Border { x_start, x_end, y: *cursor, label: None, top: false });
                *cursor += 1;
                *max_x = (*max_x).max(x_end + 2);
            }
        }
    }
}

pub(crate) fn layout(diagram: &SequenceDiagram) -> Layout {
    let columns = build_columns(&diagram.participants);
    let mut cursor = 3; // rows 0..3 hold the participant boxes
    let mut positioned = Vec::new();
    let mut active_spans = Vec::new();
    let mut active_since: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    let mut max_x = columns.last().map(|c| c.center_x + c.width / 2).unwrap_or(10);

    layout_events(
        &diagram.events,
        &columns,
        &mut cursor,
        &mut positioned,
        &mut active_spans,
        &mut active_since,
        &mut max_x,
    );

    for (id, start) in active_since {
        active_spans.push((id, start, cursor.saturating_sub(1)));
    }

    Layout {
        columns,
        positioned,
        active_spans,
        width: max_x + BLOCK_MARGIN,
        height: cursor,
    }
}

pub(crate) fn render_sequence(
    code: &str,
    theme: &crate::theme::Theme,
) -> Option<(Vec<Vec<crate::style::StyledSpan>>, usize)> {
    let diagram = parse_sequence(code)?;
    let laid_out = layout(&diagram);
    Some(render(&laid_out, theme))
}

fn render(
    laid_out: &Layout,
    theme: &crate::theme::Theme,
) -> (Vec<Vec<crate::style::StyledSpan>>, usize) {
    use super::Canvas;
    use crate::style::StyledSpan;

    let mut canvas = Canvas::new(laid_out.width, laid_out.height);
    let border_fg = Some(theme.code_border);
    let text_fg = Some(theme.fg);
    let active_fg = Some(theme.h3);

    // Lifelines first (full height) so later draws intentionally overwrite
    // the cells they cross.
    for column in &laid_out.columns {
        for y in 3..laid_out.height {
            canvas.set(column.center_x, y, '│', border_fg);
        }
    }

    for (id, y_start, y_end) in &laid_out.active_spans {
        if let Some(column) = laid_out.columns.iter().find(|c| &c.id == id) {
            let end = (*y_end).min(laid_out.height.saturating_sub(1));
            for y in *y_start..=end {
                canvas.set(column.center_x, y, '┃', active_fg);
            }
        }
    }

    for column in &laid_out.columns {
        canvas.draw_node(
            column.center_x,
            0,
            column.width,
            &column.label,
            super::NodeShape::Rectangle,
            border_fg,
            text_fg,
        );
    }

    for item in &laid_out.positioned {
        render_positioned(&mut canvas, item, border_fg, text_fg, &laid_out.columns);
    }

    let rows: Vec<Vec<StyledSpan>> = canvas.to_span_rows(theme);
    let width = laid_out.width;
    (rows, width)
}

fn render_positioned(
    canvas: &mut super::Canvas,
    item: &Positioned,
    border_fg: Option<Color>,
    text_fg: Option<Color>,
    columns: &[Column],
) {
    match item {
        Positioned::Message { from_x, to_x, label_y, arrow_y, text, line, end } => {
            if let Some(text) = text {
                draw_centered(canvas, (*from_x).min(*to_x), (*from_x).max(*to_x), *label_y, text, text_fg);
            }
            draw_h_line(canvas, *from_x, *to_x, *arrow_y, *line, *end, border_fg);
        }
        Positioned::SelfMessage { x, top_y, label_y, bottom_y, text, end } => {
            canvas.set(x + 1, *top_y, '─', border_fg);
            canvas.set(x + 2, *top_y, '╮', border_fg);
            canvas.set(x + 2, *label_y, '│', border_fg);
            if let Some(text) = text {
                for (i, c) in text.chars().enumerate() {
                    canvas.set(x + 4 + i, *label_y, c, text_fg);
                }
            }
            canvas.set(x + 2, *bottom_y, '╯', border_fg);
            canvas.set(x + 1, *bottom_y, '─', border_fg);
            let arrow = if *end == ArrowEnd::Cross { '✗' } else { '◀' };
            canvas.set(*x, *bottom_y, arrow, border_fg);
        }
        Positioned::Note { x_start, x_end, top_y, text } => {
            let width = x_end.saturating_sub(*x_start).max(4);
            canvas.draw_node(
                (x_start + x_end) / 2,
                *top_y,
                width,
                text,
                super::NodeShape::Rectangle,
                border_fg,
                text_fg,
            );
        }
        Positioned::Border { x_start, x_end, y, label, top } => {
            canvas.set(*x_start, *y, if *top { '┌' } else { '└' }, border_fg);
            canvas.set(*x_end, *y, if *top { '┐' } else { '┘' }, border_fg);
            for x in (x_start + 1)..*x_end {
                let is_lifeline = columns.iter().any(|c| c.center_x == x);
                let ch = if is_lifeline {
                    if *top { '┬' } else { '┴' }
                } else {
                    '─'
                };
                canvas.set(x, *y, ch, border_fg);
            }
            if let Some(label) = label {
                let text = format!(" {label} ");
                for (i, c) in text.chars().enumerate() {
                    canvas.set(x_start + 1 + i, *y, c, text_fg);
                }
            }
        }
    }
}

fn draw_h_line(
    canvas: &mut super::Canvas,
    from_x: usize,
    to_x: usize,
    y: usize,
    line: LineStyle,
    end: ArrowEnd,
    fg: Option<Color>,
) {
    let (left, right, forward) = if from_x <= to_x { (from_x, to_x, true) } else { (to_x, from_x, false) };
    let ch = if line == LineStyle::Dashed { '┄' } else { '─' };
    for x in (left + 1)..right {
        canvas.set(x, y, ch, fg);
    }
    let arrow = match end {
        ArrowEnd::Arrowhead => {
            if forward {
                '▶'
            } else {
                '◀'
            }
        }
        ArrowEnd::Cross => '✗',
        ArrowEnd::None => ch,
    };
    if right > left {
        if forward {
            canvas.set(right - 1, y, arrow, fg);
        } else {
            canvas.set(left + 1, y, arrow, fg);
        }
    }
}

fn draw_centered(
    canvas: &mut super::Canvas,
    left: usize,
    right: usize,
    y: usize,
    text: &str,
    fg: Option<Color>,
) {
    let mid = (left + right) / 2;
    let start = mid.saturating_sub(text.chars().count() / 2);
    for (i, c) in text.chars().enumerate() {
        canvas.set(start + i, y, c, fg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_explicit_participant_declarations() {
        let d = parse_sequence("sequenceDiagram\nparticipant A as Alice\nparticipant B\n")
            .expect("should parse");
        assert_eq!(
            d.participants,
            vec![
                Participant { id: "A".to_string(), label: "Alice".to_string() },
                Participant { id: "B".to_string(), label: "B".to_string() },
            ]
        );
    }

    #[test]
    fn actor_keyword_registers_a_participant() {
        let d = parse_sequence("sequenceDiagram\nactor U as User\n").expect("should parse");
        assert_eq!(d.participants, vec![Participant { id: "U".to_string(), label: "User".to_string() }]);
    }

    #[test]
    fn undeclared_participants_are_auto_registered_in_first_appearance_order() {
        let d = parse_sequence("sequenceDiagram\nB->>A: hi\n").expect("should parse");
        assert_eq!(
            d.participants,
            vec![
                Participant { id: "B".to_string(), label: "B".to_string() },
                Participant { id: "A".to_string(), label: "A".to_string() },
            ]
        );
    }

    #[test]
    fn parses_each_arrow_and_line_style_combination() {
        let d = parse_sequence(
            "sequenceDiagram\n\
             A->>B: solid arrowhead\n\
             A-->>B: dashed arrowhead\n\
             A->B: solid plain\n\
             A-->B: dashed plain\n\
             A-xB: solid cross\n\
             A--xB: dashed cross\n",
        )
        .expect("should parse");

        let messages: Vec<&Message> = d
            .events
            .iter()
            .map(|e| match e {
                Event::Message(m) => m,
                _ => panic!("expected only messages"),
            })
            .collect();

        assert_eq!(messages[0].line, LineStyle::Solid);
        assert_eq!(messages[0].end, ArrowEnd::Arrowhead);
        assert_eq!(messages[1].line, LineStyle::Dashed);
        assert_eq!(messages[1].end, ArrowEnd::Arrowhead);
        assert_eq!(messages[2].line, LineStyle::Solid);
        assert_eq!(messages[2].end, ArrowEnd::None);
        assert_eq!(messages[3].line, LineStyle::Dashed);
        assert_eq!(messages[3].end, ArrowEnd::None);
        assert_eq!(messages[4].line, LineStyle::Solid);
        assert_eq!(messages[4].end, ArrowEnd::Cross);
        assert_eq!(messages[5].line, LineStyle::Dashed);
        assert_eq!(messages[5].end, ArrowEnd::Cross);
    }

    #[test]
    fn message_without_text_has_none_text() {
        let d = parse_sequence("sequenceDiagram\nA->>B\n").expect("should parse");
        match &d.events[0] {
            Event::Message(m) => assert_eq!(m.text, None),
            _ => panic!("expected a message"),
        }
    }

    #[test]
    fn message_text_containing_a_colon_is_preserved() {
        let d = parse_sequence("sequenceDiagram\nA->>B: retry in: 5s\n").expect("should parse");
        match &d.events[0] {
            Event::Message(m) => assert_eq!(m.text, Some("retry in: 5s".to_string())),
            _ => panic!("expected a message"),
        }
    }

    #[test]
    fn activation_shorthand_on_arrow_sets_activate_and_deactivate() {
        let d = parse_sequence("sequenceDiagram\nA->>+B: go\nB-->>-A: done\n").expect("should parse");
        match &d.events[0] {
            Event::Message(m) => {
                assert!(m.activate);
                assert!(!m.deactivate);
                assert_eq!(m.to, "B");
            }
            _ => panic!("expected a message"),
        }
        match &d.events[1] {
            Event::Message(m) => {
                assert!(m.deactivate);
                assert!(!m.activate);
                assert_eq!(m.to, "A");
            }
            _ => panic!("expected a message"),
        }
    }

    #[test]
    fn standalone_activate_and_deactivate_statements() {
        let d = parse_sequence("sequenceDiagram\nA->>B: hi\nactivate B\ndeactivate B\n")
            .expect("should parse");
        assert_eq!(d.events[1], Event::Activate("B".to_string()));
        assert_eq!(d.events[2], Event::Deactivate("B".to_string()));
    }

    #[test]
    fn self_message_has_equal_from_and_to() {
        let d = parse_sequence("sequenceDiagram\nA->>A: think\n").expect("should parse");
        match &d.events[0] {
            Event::Message(m) => {
                assert_eq!(m.from, "A");
                assert_eq!(m.to, "A");
            }
            _ => panic!("expected a message"),
        }
    }

    #[test]
    fn parses_note_over_two_participants() {
        let d = parse_sequence("sequenceDiagram\nA->>B: hi\nNote over A,B: they meet\n")
            .expect("should parse");
        match &d.events[1] {
            Event::Note(n) => {
                assert_eq!(n.target, NoteTarget::Over("A".to_string(), Some("B".to_string())));
                assert_eq!(n.text, "they meet");
            }
            _ => panic!("expected a note"),
        }
    }

    #[test]
    fn parses_note_left_of_and_right_of() {
        let d = parse_sequence(
            "sequenceDiagram\nA->>B: hi\nNote left of A: thinking\nNote right of B: replying\n",
        )
        .expect("should parse");
        match &d.events[1] {
            Event::Note(n) => assert_eq!(n.target, NoteTarget::LeftOf("A".to_string())),
            _ => panic!("expected a note"),
        }
        match &d.events[2] {
            Event::Note(n) => assert_eq!(n.target, NoteTarget::RightOf("B".to_string())),
            _ => panic!("expected a note"),
        }
    }

    #[test]
    fn note_over_a_single_participant_registers_it_even_if_unreferenced_elsewhere() {
        let d = parse_sequence("sequenceDiagram\nNote over A: alone\n").expect("should parse");
        assert_eq!(d.participants, vec![Participant { id: "A".to_string(), label: "A".to_string() }]);
    }

    #[test]
    fn parses_nested_loop_containing_alt_else() {
        let d = parse_sequence(
            "sequenceDiagram\n\
             loop Every request\n\
             alt happy path\n\
             A->>B: ok\n\
             else failure\n\
             A->>B: retry\n\
             end\n\
             end\n",
        )
        .expect("should parse");

        assert_eq!(d.events.len(), 1);
        let outer = match &d.events[0] {
            Event::Block(b) => b,
            _ => panic!("expected a block"),
        };
        assert_eq!(outer.sections.len(), 1);
        assert_eq!(outer.sections[0].label, "loop Every request");
        assert_eq!(outer.sections[0].events.len(), 1);

        let inner = match &outer.sections[0].events[0] {
            Event::Block(b) => b,
            _ => panic!("expected a nested block"),
        };
        assert_eq!(inner.sections.len(), 2);
        assert_eq!(inner.sections[0].label, "alt happy path");
        assert_eq!(inner.sections[1].label, "else failure");
        assert_eq!(inner.sections[0].events.len(), 1);
        assert_eq!(inner.sections[1].events.len(), 1);
    }

    #[test]
    fn parses_par_with_and_divider() {
        let d = parse_sequence(
            "sequenceDiagram\npar fetch A\nX->>Y: a\nand fetch B\nX->>Y: b\nend\n",
        )
        .expect("should parse");
        let block = match &d.events[0] {
            Event::Block(b) => b,
            _ => panic!("expected a block"),
        };
        assert_eq!(block.sections[0].label, "par fetch A");
        assert_eq!(block.sections[1].label, "and fetch B");
    }

    #[test]
    fn comments_and_unrecognized_directives_are_skipped_without_creating_participants() {
        let d = parse_sequence(
            "sequenceDiagram\n%% a comment\nautonumber\ntitle Some Title\nA->>B: hi\n",
        )
        .expect("should parse");
        assert_eq!(d.participants.len(), 2);
        assert_eq!(d.events.len(), 1);
    }

    #[test]
    fn non_sequence_diagram_input_returns_none() {
        assert_eq!(parse_sequence("graph TD\nA-->B\n"), None);
    }

    #[test]
    fn empty_sequence_diagram_returns_none() {
        assert_eq!(parse_sequence("sequenceDiagram\n"), None);
    }

    fn column_x(layout: &Layout, id: &str) -> usize {
        layout
            .columns
            .iter()
            .find(|c| c.id == id)
            .map(|c| c.center_x)
            .unwrap_or_else(|| panic!("no column for participant {id}"))
    }

    #[test]
    fn columns_are_laid_out_left_to_right_in_declaration_order() {
        let d = parse_sequence("sequenceDiagram\nparticipant A\nparticipant B\nparticipant C\n")
            .expect("should parse");
        let l = layout(&d);
        assert_eq!(l.columns.len(), 3);
        assert!(column_x(&l, "A") < column_x(&l, "B"));
        assert!(column_x(&l, "B") < column_x(&l, "C"));
    }

    #[test]
    fn message_row_assigns_label_then_arrow_row_below_it() {
        let d = parse_sequence("sequenceDiagram\nA->>B: hi\nA->>B: again\n").expect("should parse");
        let l = layout(&d);
        let rows: Vec<(usize, usize)> = l
            .positioned
            .iter()
            .filter_map(|p| match p {
                Positioned::Message { label_y, arrow_y, .. } => Some((*label_y, *arrow_y)),
                _ => None,
            })
            .collect();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].1, rows[0].0 + 1, "arrow row is directly below the label row");
        assert!(rows[1].0 > rows[0].1, "second message starts after the first one's arrow row");
    }

    #[test]
    fn self_message_uses_three_rows() {
        let d = parse_sequence("sequenceDiagram\nA->>A: think\n").expect("should parse");
        let l = layout(&d);
        match &l.positioned[0] {
            Positioned::SelfMessage { top_y, label_y, bottom_y, .. } => {
                assert_eq!(*label_y, *top_y + 1);
                assert_eq!(*bottom_y, *top_y + 2);
            }
            _ => panic!("expected a self-message"),
        }
    }

    #[test]
    fn note_over_two_participants_spans_between_their_columns() {
        let d = parse_sequence("sequenceDiagram\nA->>B: hi\nNote over A,B: both\n")
            .expect("should parse");
        let l = layout(&d);
        let ax = column_x(&l, "A");
        let bx = column_x(&l, "B");
        match l.positioned.iter().find(|p| matches!(p, Positioned::Note { .. })).unwrap() {
            Positioned::Note { x_start, x_end, .. } => {
                assert!(*x_start <= ax.min(bx));
                assert!(*x_end >= ax.max(bx));
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn block_span_covers_only_the_participants_it_references() {
        let d = parse_sequence(
            "sequenceDiagram\nparticipant A\nparticipant B\nparticipant C\nloop x\nA->>B: hi\nend\n",
        )
        .expect("should parse");
        let l = layout(&d);
        let ax = column_x(&l, "A");
        let bx = column_x(&l, "B");
        let cx = column_x(&l, "C");

        let borders: Vec<&Positioned> =
            l.positioned.iter().filter(|p| matches!(p, Positioned::Border { .. })).collect();
        assert_eq!(borders.len(), 2, "one top + one bottom border for a single-section loop");
        match borders[0] {
            Positioned::Border { x_start, x_end, .. } => {
                assert!(*x_start <= ax.min(bx));
                assert!(*x_end >= ax.max(bx));
                assert!(*x_end < cx, "block must not extend to C, which it never references");
            }
            _ => unreachable!(),
        }
    }

    #[test]
    fn nested_alt_else_produces_top_divider_and_bottom_borders() {
        let d = parse_sequence(
            "sequenceDiagram\nalt happy\nA->>B: ok\nelse sad\nA->>B: retry\nend\n",
        )
        .expect("should parse");
        let l = layout(&d);
        let borders: Vec<&Positioned> =
            l.positioned.iter().filter(|p| matches!(p, Positioned::Border { .. })).collect();
        // top border ("alt happy"), divider ("else sad"), bottom border.
        assert_eq!(borders.len(), 3);
    }

    #[test]
    fn activation_span_runs_from_activate_to_deactivate() {
        // Mermaid's `-` shorthand deactivates the SOURCE of that arrow, not
        // the destination it's textually adjacent to: in `B-->>-A`, it's B
        // (the source, already active from the earlier `+B`) that
        // deactivates, even though `-` sits right before `A`. This mirrors
        // real Mermaid's own "Alice->>+John: ...\nJohn-->>-Alice: ..." example,
        // where John (not Alice) is the one whose activation bar ends.
        let d = parse_sequence("sequenceDiagram\nA->>+B: go\nB->>B: work\nB-->>-A: done\n")
            .expect("should parse");
        let l = layout(&d);
        assert_eq!(l.active_spans, vec![("B".to_string(), 5, 10)]);
    }

    #[test]
    fn activation_never_deactivated_stays_active_through_diagram_end() {
        let d = parse_sequence("sequenceDiagram\nA->>+B: go\n").expect("should parse");
        let l = layout(&d);
        assert_eq!(l.active_spans.len(), 1);
        assert_eq!(l.active_spans[0].0, "B");
        assert_eq!(l.active_spans[0].2, l.height - 1);
    }

    fn flatten(rows: &[Vec<crate::style::StyledSpan>]) -> String {
        rows.iter()
            .map(|row| row.iter().map(|s| s.text.as_str()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn renders_participants_messages_and_note() {
        let theme = crate::theme::Theme::dark();
        let code = "sequenceDiagram\n\
                     participant A as Alice\n\
                     participant B as Bob\n\
                     A->>B: hello\n\
                     B-->>A: hi back\n\
                     Note over A,B: they are friends\n";
        let (rows, width) = render_sequence(code, &theme).expect("should render");
        let text = flatten(&rows);
        assert!(text.contains("Alice"), "expected participant label in:\n{text}");
        assert!(text.contains("Bob"), "expected participant label in:\n{text}");
        assert!(text.contains("hello"), "expected message text in:\n{text}");
        assert!(text.contains("hi back"), "expected message text in:\n{text}");
        assert!(text.contains("they are friends"), "expected note text in:\n{text}");
        assert!(text.contains('▶'), "expected a solid arrowhead for ->>");
        assert!(width > 0);
    }

    #[test]
    fn renders_loop_block_border_with_label() {
        let theme = crate::theme::Theme::dark();
        let code = "sequenceDiagram\nA->>B: hi\nloop Every request\nA->>B: poll\nend\n";
        let (rows, _width) = render_sequence(code, &theme).expect("should render");
        let text = flatten(&rows);
        assert!(text.contains("loop Every request"), "expected loop label in:\n{text}");
    }

    #[test]
    fn non_sequence_input_still_falls_through_to_none() {
        let theme = crate::theme::Theme::dark();
        assert!(render_sequence("graph TD\nA-->B\n", &theme).is_none());
    }
}
