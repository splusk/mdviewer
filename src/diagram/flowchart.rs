use crate::style::StyledSpan;
use crate::theme::Theme;
use std::collections::{HashMap, VecDeque};

use super::{Canvas, NodeShape};

// ───── Data types ─────

#[derive(Debug, Clone, Copy, PartialEq)]
enum Direction {
    TopDown,
    LeftRight,
}

#[derive(Debug, Clone)]
struct Node {
    label: String,
    shape: NodeShape,
}

#[derive(Debug, Clone)]
struct Edge {
    from: String,
    to: String,
    label: Option<String>,
}

#[derive(Debug)]
struct Graph {
    direction: Direction,
    nodes: HashMap<String, Node>,
    edges: Vec<Edge>,
    node_order: Vec<String>,
}

// ───── Parser ─────

fn parse_flowchart(code: &str) -> Option<Graph> {
    let mut direction = Direction::TopDown;
    let mut nodes: HashMap<String, Node> = HashMap::new();
    let mut edges: Vec<Edge> = Vec::new();
    let mut node_order: Vec<String> = Vec::new();

    for line in code.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("%%") {
            continue;
        }

        // Direction declaration
        if trimmed.starts_with("graph ") || trimmed.starts_with("flowchart ") {
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            if parts.len() >= 2 {
                direction = match parts[1] {
                    "LR" | "RL" => Direction::LeftRight,
                    _ => Direction::TopDown,
                };
            }
            continue;
        }

        // Skip unsupported directives
        if trimmed.starts_with("subgraph")
            || trimmed == "end"
            || trimmed.starts_with("style ")
            || trimmed.starts_with("classDef ")
            || trimmed.starts_with("class ")
            || trimmed.starts_with("linkStyle ")
            || trimmed.starts_with("click ")
        {
            continue;
        }

        parse_line(trimmed, &mut nodes, &mut edges, &mut node_order);
    }

    if nodes.is_empty() {
        return None;
    }

    Some(Graph {
        direction,
        nodes,
        edges,
        node_order,
    })
}

#[allow(clippy::type_complexity)]
fn parse_node_ref(s: &str) -> Option<(String, Option<(String, NodeShape)>, &str)> {
    let s = s.trim_start();
    if s.is_empty() {
        return None;
    }

    // Extract node ID (alphanumeric, underscore, hyphen)
    let id_end = s
        .find(|c: char| !c.is_alphanumeric() && c != '_' && c != '-')
        .unwrap_or(s.len());
    if id_end == 0 {
        return None;
    }
    let id = s[..id_end].to_string();
    let rest = &s[id_end..];

    // Double parens: ((label))
    if rest.starts_with("((")
        && let Some(end) = rest.find("))")
    {
        let label = rest[2..end].trim().to_string();
        return Some((id, Some((label, NodeShape::Circle)), &rest[end + 2..]));
    }

    // Square brackets: [label]
    if rest.starts_with('[')
        && let Some(end) = find_matching(rest, '[', ']')
    {
        let label = rest[1..end].trim().to_string();
        return Some((id, Some((label, NodeShape::Rectangle)), &rest[end + 1..]));
    }

    // Curly braces: {label}
    if rest.starts_with('{')
        && let Some(end) = find_matching(rest, '{', '}')
    {
        let label = rest[1..end].trim().to_string();
        return Some((id, Some((label, NodeShape::Diamond)), &rest[end + 1..]));
    }

    // Parentheses: (label)
    if rest.starts_with('(')
        && let Some(end) = find_matching(rest, '(', ')')
    {
        let label = rest[1..end].trim().to_string();
        return Some((id, Some((label, NodeShape::Rounded)), &rest[end + 1..]));
    }

    Some((id, None, rest))
}

fn find_matching(s: &str, open: char, close: char) -> Option<usize> {
    let mut depth: usize = 0;
    for (i, c) in s.char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            if depth == 0 {
                continue;
            }
            depth -= 1;
            if depth == 0 {
                return Some(i);
            }
        }
    }
    None
}

fn register_node(
    id: &str,
    label_shape: Option<(String, NodeShape)>,
    nodes: &mut HashMap<String, Node>,
    node_order: &mut Vec<String>,
) {
    if let Some(node) = nodes.get_mut(id) {
        if let Some((label, shape)) = label_shape {
            node.label = label;
            node.shape = shape;
        }
    } else {
        let (label, shape) = label_shape.unwrap_or_else(|| (id.to_string(), NodeShape::Rectangle));
        nodes.insert(id.to_string(), Node { label, shape });
        node_order.push(id.to_string());
    }
}

fn parse_line(
    line: &str,
    nodes: &mut HashMap<String, Node>,
    edges: &mut Vec<Edge>,
    node_order: &mut Vec<String>,
) {
    let (first_id, first_label, mut remaining) = match parse_node_ref(line) {
        Some(r) => r,
        None => return,
    };
    register_node(&first_id, first_label, nodes, node_order);

    let mut prev_id = first_id;

    // Parse chain of edges: A --> B --> C
    loop {
        let trimmed = remaining.trim_start();
        if trimmed.is_empty() {
            break;
        }

        let (edge_label, arrow_rest) = match parse_arrow(trimmed) {
            Some(r) => r,
            None => break,
        };

        remaining = arrow_rest;

        let (next_id, next_label, rest) = match parse_node_ref(remaining) {
            Some(r) => r,
            None => break,
        };
        register_node(&next_id, next_label, nodes, node_order);

        edges.push(Edge {
            from: prev_id.clone(),
            to: next_id.clone(),
            label: edge_label,
        });

        prev_id = next_id;
        remaining = rest;
    }
}

fn parse_arrow(s: &str) -> Option<(Option<String>, &str)> {
    let s = s.trim_start();

    // "-- label -->" syntax
    if s.starts_with("-- ")
        && let Some(arrow_pos) = s[3..].find("-->")
    {
        let label = s[3..3 + arrow_pos].trim().to_string();
        let rest = &s[3 + arrow_pos + 3..];
        return Some((Some(label), rest));
    }

    // Standard arrows
    let arrows = ["--->", "-->", "---", "-.->", "==>"];
    for arrow in &arrows {
        if let Some(rest) = s.strip_prefix(arrow) {
            // Check for |label| after arrow
            let trimmed_rest = rest.trim_start();
            if trimmed_rest.starts_with('|')
                && let Some(end) = trimmed_rest[1..].find('|')
            {
                let label = trimmed_rest[1..1 + end].trim().to_string();
                return Some((Some(label), &trimmed_rest[2 + end..]));
            }
            return Some((None, rest));
        }
    }

    None
}

// ───── Layout ─────

#[derive(Clone)]
pub(crate) struct NodeLayout {
    pub(crate) center_x: usize,
    pub(crate) top_y: usize,
    pub(crate) width: usize,
}

fn assign_layers(graph: &Graph) -> Vec<Vec<String>> {
    // Build adjacency and in-degree
    let mut in_degree: HashMap<&str, usize> = HashMap::new();
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();

    for id in graph.nodes.keys() {
        in_degree.entry(id.as_str()).or_insert(0);
        adj.entry(id.as_str()).or_default();
    }

    for edge in &graph.edges {
        adj.entry(edge.from.as_str())
            .or_default()
            .push(edge.to.as_str());
        *in_degree.entry(edge.to.as_str()).or_insert(0) += 1;
    }

    // Kahn's topological sort
    let mut queue: VecDeque<&str> = VecDeque::new();
    let mut topo_order: Vec<String> = Vec::new();
    let mut in_deg = in_degree.clone();

    for (&id, &deg) in &in_deg {
        if deg == 0 {
            queue.push_back(id);
        }
    }

    // Cycle fallback
    if queue.is_empty()
        && let Some(first) = graph.node_order.first()
    {
        queue.push_back(first.as_str());
    }

    while let Some(node) = queue.pop_front() {
        topo_order.push(node.to_string());
        if let Some(neighbors) = adj.get(node) {
            for &next in neighbors {
                let deg = in_deg.get_mut(next).unwrap();
                *deg = deg.saturating_sub(1);
                if *deg == 0 {
                    queue.push_back(next);
                }
            }
        }
    }

    // Add any remaining nodes (from cycles)
    for id in &graph.node_order {
        if !topo_order.contains(id) {
            topo_order.push(id.clone());
        }
    }

    // Longest-path layer assignment
    let mut node_layer: HashMap<String, usize> = HashMap::new();
    for node in &topo_order {
        let mut max_parent_layer: Option<usize> = None;
        for edge in &graph.edges {
            if edge.to == *node
                && let Some(&parent_layer) = node_layer.get(&edge.from)
            {
                max_parent_layer =
                    Some(max_parent_layer.map_or(parent_layer, |m: usize| m.max(parent_layer)));
            }
        }
        let layer = max_parent_layer.map_or(0, |m| m + 1);
        node_layer.insert(node.clone(), layer);
    }

    let max_layer = node_layer.values().copied().max().unwrap_or(0);
    let mut layers: Vec<Vec<String>> = vec![Vec::new(); max_layer + 1];
    for node in &topo_order {
        let layer = node_layer[node];
        layers[layer].push(node.clone());
    }
    layers.retain(|l| !l.is_empty());
    layers
}

fn order_within_layers(layers: &mut [Vec<String>], graph: &Graph) {
    // Barycenter heuristic to reduce edge crossings
    for _ in 0..4 {
        // Forward pass
        for i in 1..layers.len() {
            let prev_layer = layers[i - 1].clone();
            let mut positions: Vec<(String, f64)> = Vec::new();

            for node in &layers[i] {
                let mut parent_positions: Vec<f64> = Vec::new();
                for edge in &graph.edges {
                    if edge.to == *node
                        && let Some(pos) = prev_layer.iter().position(|n| n == &edge.from)
                    {
                        parent_positions.push(pos as f64);
                    }
                }
                let avg = if parent_positions.is_empty() {
                    layers[i].iter().position(|n| n == node).unwrap_or(0) as f64
                } else {
                    parent_positions.iter().sum::<f64>() / parent_positions.len() as f64
                };
                positions.push((node.clone(), avg));
            }
            positions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            layers[i] = positions.into_iter().map(|(n, _)| n).collect();
        }

        // Backward pass
        for i in (0..layers.len().saturating_sub(1)).rev() {
            let next_layer = layers[i + 1].clone();
            let mut positions: Vec<(String, f64)> = Vec::new();

            for node in &layers[i] {
                let mut child_positions: Vec<f64> = Vec::new();
                for edge in &graph.edges {
                    if edge.from == *node
                        && let Some(pos) = next_layer.iter().position(|n| n == &edge.to)
                    {
                        child_positions.push(pos as f64);
                    }
                }
                let avg = if child_positions.is_empty() {
                    layers[i].iter().position(|n| n == node).unwrap_or(0) as f64
                } else {
                    child_positions.iter().sum::<f64>() / child_positions.len() as f64
                };
                positions.push((node.clone(), avg));
            }
            positions.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
            layers[i] = positions.into_iter().map(|(n, _)| n).collect();
        }
    }
}

fn node_box_width(node: &Node) -> usize {
    label_box_width(&node.label, node.shape)
}

pub(crate) fn label_box_width(label: &str, shape: NodeShape) -> usize {
    let label_width = label.chars().count();
    let width = match shape {
        NodeShape::Diamond => label_width + 6,
        _ => label_width + 4,
    };
    width.max(7)
}

// ───── Top-Down rendering ─────

fn render_td(graph: &Graph, theme: &Theme) -> Option<(Vec<Vec<StyledSpan>>, usize)> {
    let node_height: usize = 3;
    let edge_gap: usize = 4;
    let h_gap: usize = 4;

    let mut layers = assign_layers(graph);
    order_within_layers(&mut layers, graph);

    // Calculate node widths
    let mut widths: HashMap<String, usize> = HashMap::new();
    for (id, node) in &graph.nodes {
        widths.insert(id.clone(), node_box_width(node));
    }

    // Find widest layer to determine canvas width
    let mut max_layer_width: usize = 0;
    for layer in &layers {
        let w: usize = layer
            .iter()
            .map(|id| widths.get(id).copied().unwrap_or(7))
            .sum::<usize>()
            + layer.len().saturating_sub(1) * h_gap;
        max_layer_width = max_layer_width.max(w);
    }

    let canvas_width = max_layer_width + 6; // margin on each side
    let canvas_height = layers.len() * (node_height + edge_gap) - edge_gap;

    if canvas_height == 0 {
        return None;
    }

    let mut canvas = Canvas::new(canvas_width, canvas_height);

    // Calculate node positions and draw nodes
    let mut positions: HashMap<String, NodeLayout> = HashMap::new();
    let border_fg = Some(theme.code_border);
    let text_fg = Some(theme.fg);

    // First pass: calculate centers for the widest layer
    // Then align single-node layers to the canvas center
    let canvas_center = canvas_width / 2;

    for (layer_idx, layer) in layers.iter().enumerate() {
        let y = layer_idx * (node_height + edge_gap);

        // Compute node centers relative to layer, then offset to center in canvas
        let node_widths_in_layer: Vec<usize> = layer
            .iter()
            .map(|id| widths.get(id).copied().unwrap_or(7))
            .collect();
        let layer_width: usize =
            node_widths_in_layer.iter().sum::<usize>() + layer.len().saturating_sub(1) * h_gap;

        // Compute center of each node within the layer
        let mut centers_in_layer: Vec<usize> = Vec::new();
        let mut cumulative = 0;
        for &w in &node_widths_in_layer {
            centers_in_layer.push(cumulative + w / 2);
            cumulative += w + h_gap;
        }

        // Center of the layer
        let layer_center = if layer_width > 0 { layer_width / 2 } else { 0 };

        for (i, id) in layer.iter().enumerate() {
            let w = node_widths_in_layer[i];
            // Shift node center so that the layer center aligns with canvas center
            let cx = (canvas_center as isize + centers_in_layer[i] as isize - layer_center as isize)
                .max(w as isize / 2) as usize;

            if let Some(node) = graph.nodes.get(id) {
                canvas.draw_node(cx, y, w, &node.label, node.shape, border_fg, text_fg);
            }

            positions.insert(
                id.clone(),
                NodeLayout {
                    center_x: cx,
                    top_y: y,
                    width: w,
                },
            );
        }
    }

    // Draw edges
    let edge_fg = Some(theme.code_border);
    let label_fg = Some(theme.h3); // Use a distinct color for edge labels

    for edge in &graph.edges {
        if let (Some(src), Some(dst)) = (positions.get(&edge.from), positions.get(&edge.to)) {
            let src_bottom = src.top_y + 2;
            let dst_top = dst.top_y;
            canvas.draw_edge_td(
                src.center_x,
                src_bottom,
                dst.center_x,
                dst_top,
                edge.label.as_deref(),
                edge_fg,
                label_fg,
            );
        }
    }

    let rows = canvas.to_span_rows(theme);
    Some((rows, canvas_width))
}

// ───── Left-Right rendering ─────

fn render_lr(graph: &Graph, theme: &Theme) -> Option<(Vec<Vec<StyledSpan>>, usize)> {
    let node_height: usize = 3;
    let node_h_gap: usize = 6; // horizontal gap between columns for edge routing
    let v_gap: usize = 2; // vertical gap between nodes in same column

    let mut layers = assign_layers(graph);
    order_within_layers(&mut layers, graph);

    // Calculate node widths
    let mut widths: HashMap<String, usize> = HashMap::new();
    for (id, node) in &graph.nodes {
        widths.insert(id.clone(), node_box_width(node));
    }

    // Column widths (max node width per layer)
    let col_widths: Vec<usize> = layers
        .iter()
        .map(|layer| {
            layer
                .iter()
                .map(|id| widths.get(id).copied().unwrap_or(7))
                .max()
                .unwrap_or(7)
        })
        .collect();

    let max_nodes_in_layer = layers.iter().map(|l| l.len()).max().unwrap_or(1);

    let canvas_width: usize =
        col_widths.iter().sum::<usize>() + (layers.len().saturating_sub(1)) * node_h_gap + 4;
    let canvas_height = max_nodes_in_layer * (node_height + v_gap) - v_gap + 2;

    if canvas_height == 0 {
        return None;
    }

    let mut canvas = Canvas::new(canvas_width, canvas_height);

    let mut positions: HashMap<String, NodeLayout> = HashMap::new();
    let border_fg = Some(theme.code_border);
    let text_fg = Some(theme.fg);

    let mut col_x = 2; // starting x with margin
    for (layer_idx, layer) in layers.iter().enumerate() {
        let col_w = col_widths[layer_idx];

        let total_layer_height = layer.len() * node_height + layer.len().saturating_sub(1) * v_gap;
        let start_y = (canvas_height.saturating_sub(total_layer_height)) / 2;

        for (node_idx, id) in layer.iter().enumerate() {
            let w = widths.get(id).copied().unwrap_or(7);
            let cx = col_x + col_w / 2;
            let y = start_y + node_idx * (node_height + v_gap);

            if let Some(node) = graph.nodes.get(id) {
                canvas.draw_node(cx, y, w, &node.label, node.shape, border_fg, text_fg);
            }

            positions.insert(
                id.clone(),
                NodeLayout {
                    center_x: cx,
                    top_y: y,
                    width: w,
                },
            );
        }

        col_x += col_w + node_h_gap;
    }

    // Draw edges
    let edge_fg = Some(theme.code_border);
    let label_fg = Some(theme.h3);

    for edge in &graph.edges {
        if let (Some(src), Some(dst)) = (positions.get(&edge.from), positions.get(&edge.to)) {
            let src_right_x = src.center_x + src.width / 2;
            let src_cy = src.top_y + 1;
            let dst_left_x = dst.center_x.saturating_sub(dst.width / 2);
            let dst_cy = dst.top_y + 1;

            canvas.draw_edge_lr(
                src.center_x,
                src_right_x,
                src_cy,
                dst_left_x,
                dst_cy,
                edge.label.as_deref(),
                edge_fg,
                label_fg,
                None,
            );
        }
    }

    let rows = canvas.to_span_rows(theme);
    Some((rows, canvas_width))
}

// ───── Public API ─────

/// Try to render mermaid code as a visual diagram.
/// Returns (content_rows, content_width) or None if parsing fails.
pub(crate) fn render_flowchart(code: &str, theme: &Theme) -> Option<(Vec<Vec<StyledSpan>>, usize)> {
    let graph = parse_flowchart(code)?;
    match graph.direction {
        Direction::TopDown => render_td(&graph, theme),
        Direction::LeftRight => render_lr(&graph, theme),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::Theme;

    fn flatten(rows: &[Vec<StyledSpan>]) -> String {
        rows.iter()
            .map(|row| row.iter().map(|s| s.text.as_str()).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn renders_simple_top_down_flowchart() {
        let theme = Theme::dark();
        let (rows, width) =
            render_flowchart("graph TD\nA[Start] --> B[End]", &theme).expect("flowchart should render");
        let text = flatten(&rows);
        assert!(text.contains("Start"), "expected 'Start' node in:\n{text}");
        assert!(text.contains("End"), "expected 'End' node in:\n{text}");
        assert!(width > 0);
    }

    #[test]
    fn renders_left_right_flowchart_with_edge_label() {
        let theme = Theme::dark();
        let (rows, _width) = render_flowchart("graph LR\nA[One] -- go --> B[Two]", &theme)
            .expect("flowchart should render");
        let text = flatten(&rows);
        assert!(text.contains("One"), "expected 'One' node in:\n{text}");
        assert!(text.contains("Two"), "expected 'Two' node in:\n{text}");
        assert!(text.contains("go"), "expected edge label 'go' in:\n{text}");
    }

    #[test]
    fn renders_diamond_and_circle_shapes() {
        let theme = Theme::dark();
        let (rows, _width) =
            render_flowchart("graph TD\nA{Decision} --> B((Circle))", &theme).expect("should render");
        let text = flatten(&rows);
        assert!(text.contains("Decision"));
        assert!(text.contains("Circle"));
        assert!(text.contains('◆'), "diamond shape should use ◆ border chars");
    }

    #[test]
    fn unparseable_input_returns_none() {
        // The flowchart parser is lenient: any line starting with an
        // alphanumeric/underscore/hyphen char is treated as a node id, so
        // plain prose (e.g. "not a mermaid diagram at all") actually parses
        // as a one-node graph rather than returning None. Use input whose
        // every line fails `parse_node_ref` (starts with punctuation) to
        // exercise the real None path.
        let theme = Theme::dark();
        assert!(render_flowchart("!!! definitely not mermaid !!!", &theme).is_none());
    }

    #[test]
    fn empty_code_block_returns_none() {
        let theme = Theme::dark();
        assert!(render_flowchart("", &theme).is_none());
    }
}
