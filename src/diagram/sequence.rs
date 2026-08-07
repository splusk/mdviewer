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

pub(crate) fn render_sequence(
    code: &str,
    _theme: &crate::theme::Theme,
) -> Option<(Vec<Vec<crate::style::StyledSpan>>, usize)> {
    let _diagram = parse_sequence(code)?;
    None
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
}
