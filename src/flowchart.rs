use crate::layout::{self, Groups, LayeredOptions};
use crate::{PeleError, RenderOptions, RenderResult, svg, text};
use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Direction {
    #[default]
    Tb,
    Bt,
    Lr,
    Rl,
}

impl Direction {
    fn parse(value: &str) -> Option<Self> {
        match value.trim() {
            "TD" | "TB" => Some(Self::Tb),
            "BT" => Some(Self::Bt),
            "LR" => Some(Self::Lr),
            "RL" => Some(Self::Rl),
            _ => None,
        }
    }

    fn sideways(self) -> bool {
        matches!(self, Self::Lr | Self::Rl)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Shape {
    #[default]
    Rect,
    Rounded,
    Stadium,
    Subroutine,
    Cylinder,
    Circle,
    DoubleCircle,
    Ellipse,
    Diamond,
    Hexagon,
    LeanRight,
    LeanLeft,
    TrapBottom,
    TrapTop,
    Asymmetric,
    Text,
    Start,
    End,
    Fork,
    Extended(&'static str),
}

impl Shape {
    fn named(value: &str) -> Option<Self> {
        Some(match value.trim() {
            "rect" | "proc" | "process" | "rectangle" | "squareRect" => Self::Rect,
            "rounded" | "event" | "roundedRect" => Self::Rounded,
            "stadium" | "terminal" | "pill" => Self::Stadium,
            "fr-rect" | "subprocess" | "subproc" | "framed-rectangle" | "subroutine" => {
                Self::Subroutine
            }
            "cyl" | "db" | "database" | "cylinder" => Self::Cylinder,
            "circle" | "circ" => Self::Circle,
            "dbl-circ" | "double-circle" | "doublecircle" => Self::DoubleCircle,
            "ellipse" => Self::Ellipse,
            "diam" | "decision" | "diamond" | "question" => Self::Diamond,
            "hex" | "hexagon" | "prepare" => Self::Hexagon,
            "lean-r" | "lean-right" | "in-out" | "lean_right" => Self::LeanRight,
            "lean-l" | "lean-left" | "out-in" | "lean_left" => Self::LeanLeft,
            "trap-b" | "priority" | "trapezoid-bottom" | "trapezoid" => Self::TrapBottom,
            "trap-t" | "manual" | "trapezoid-top" | "inv-trapezoid" | "inv_trapezoid" => {
                Self::TrapTop
            }
            "odd" | "rect_left_inv_arrow" => Self::Asymmetric,
            "text" => Self::Text,
            "sm-circ" | "start" | "small-circle" | "stateStart" => Self::Start,
            "fr-circ" | "stop" | "framed-circle" | "stateEnd" => Self::End,
            "fork" | "join" | "forkJoin" => Self::Fork,
            _ => Self::Extended(crate::flow_shapes::canonical(value)?),
        })
    }

    fn size(self, width: f64, height: f64) -> (f64, f64, f64) {
        let base_h = height + 24.0;
        match self {
            Self::Rect | Self::Rounded => ((width + 36.0).max(48.0), base_h, 0.0),
            Self::Stadium => (width + base_h * 0.5 + 28.0, base_h, 0.0),
            Self::Subroutine => (width + 52.0, base_h, 0.0),
            Self::Cylinder => {
                let w = (width + 36.0).max(56.0);
                let ry = (w / 14.0).clamp(6.0, 12.0);
                (w, base_h + 2.0 * ry, ry / 2.0)
            }
            Self::Circle | Self::DoubleCircle => {
                let extra = if self == Self::Circle { 16.0 } else { 26.0 };
                let side = width.hypot(height).max(28.0) + extra;
                (side, side, 0.0)
            }
            Self::Ellipse => (width * 1.3 + 36.0, height * 1.2 + 24.0, 0.0),
            Self::Diamond => {
                let a = width + 16.0;
                let b = height + 8.0;
                (a + 1.6 * b, a / 1.6 + b, 0.0)
            }
            Self::Hexagon | Self::LeanRight | Self::LeanLeft | Self::TrapBottom | Self::TrapTop => {
                (width + 36.0 + base_h / 2.0, base_h, 0.0)
            }
            Self::Asymmetric => (width + 36.0 + base_h / 4.0, base_h, 0.0),
            Self::Text => (width + 8.0, height + 8.0, 0.0),
            Self::Start => (14.0, 14.0, 0.0),
            Self::End => (20.0, 20.0, 0.0),
            Self::Fork => (70.0, 8.0, 0.0),
            Self::Extended(name) => {
                crate::flow_shapes::size(name, width, height).expect("canonical extended shape")
            }
        }
    }

    fn has_label(self) -> bool {
        match self {
            Self::Start | Self::End | Self::Fork => false,
            Self::Extended(name) => crate::flow_shapes::has_label(name),
            _ => true,
        }
    }

    fn span(self, w: f64, h: f64, sideways: bool) -> f64 {
        let side = if sideways { h } else { w };
        match self {
            Self::Rect | Self::Rounded | Self::Subroutine | Self::Text => side - 24.0,
            Self::Circle
            | Self::DoubleCircle
            | Self::Ellipse
            | Self::Start
            | Self::End
            | Self::Diamond => side * 0.6,
            Self::Stadium => {
                if sideways {
                    h * 0.6
                } else {
                    w - h
                }
            }
            Self::Cylinder => {
                if sideways {
                    h - 36.0
                } else {
                    w * 0.6
                }
            }
            Self::Hexagon => {
                if sideways {
                    h * 0.5
                } else {
                    w - h / 2.0 - 12.0
                }
            }
            Self::LeanRight | Self::LeanLeft | Self::TrapBottom | Self::TrapTop => {
                if sideways {
                    0.0
                } else {
                    w - h / 2.0 - 24.0
                }
            }
            Self::Asymmetric => {
                if sideways {
                    0.0
                } else {
                    w - 24.0
                }
            }
            Self::Fork => 0.0,
            Self::Extended(name) => crate::flow_shapes::span(name, w, h, sideways),
        }
        .max(0.0)
    }

    fn inset(self, w: f64, h: f64, side: usize, t: f64) -> f64 {
        let odd = !side.is_multiple_of(2);
        let bulge = |half: f64, depth: f64| {
            let k = (t.abs() / half).min(1.0);
            depth * (1.0 - (1.0 - k * k).sqrt())
        };
        match self {
            Self::Circle | Self::DoubleCircle | Self::Ellipse | Self::Start | Self::End => {
                if odd {
                    bulge(h / 2.0, w / 2.0)
                } else {
                    bulge(w / 2.0, h / 2.0)
                }
            }
            Self::Stadium => {
                if odd {
                    bulge(h / 2.0, h / 2.0)
                } else {
                    0.0
                }
            }
            Self::Cylinder => {
                if odd {
                    0.0
                } else {
                    bulge(w / 2.0, (w / 14.0).clamp(6.0, 12.0))
                }
            }
            Self::Diamond => {
                if odd {
                    t.abs() * w / h
                } else {
                    t.abs() * h / w
                }
            }
            Self::Hexagon => {
                if odd {
                    t.abs() / 2.0
                } else {
                    0.0
                }
            }
            Self::LeanRight | Self::LeanLeft | Self::TrapBottom | Self::TrapTop => {
                if odd {
                    h / 8.0
                } else {
                    0.0
                }
            }
            Self::Asymmetric => {
                if side == 3 {
                    h / 4.0
                } else {
                    0.0
                }
            }
            Self::Extended(name) => crate::flow_shapes::inset(name, w, h, side, t),
            _ => 0.0,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Marker {
    #[default]
    None,
    Point,
    Circle,
    Cross,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum Stroke {
    #[default]
    Normal,
    Dotted,
    Thick,
    Invisible,
}

#[derive(Clone, Debug)]
struct Node {
    id: String,
    label: String,
    shape: Shape,
    styles: Vec<String>,
    classes: Vec<String>,
    parent: Option<usize>,
    link: Option<String>,
    target: Option<String>,
    tooltip: Option<String>,
}

#[derive(Clone, Debug)]
struct Edge {
    id: String,
    tail: usize,
    head: usize,
    label: String,
    stroke: Stroke,
    start: Marker,
    end: Marker,
    minlen: usize,
    styles: Vec<String>,
    classes: Vec<String>,
}

#[derive(Clone, Debug)]
struct Group {
    id: String,
    title: String,
    parent: Option<usize>,
    direction: Option<Direction>,
    styles: Vec<String>,
    classes: Vec<String>,
}

#[derive(Clone, Debug, Default)]
struct Model {
    direction: Direction,
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    groups: Vec<Group>,
    classes: HashMap<String, Vec<String>>,
    default_edge_style: Vec<String>,
    acc_title: Option<String>,
    acc_descr: Option<String>,
}

#[derive(Clone, Debug)]
struct Statement {
    text: String,
    line: usize,
    column: usize,
}

fn statements(source: &str) -> Result<Vec<Statement>, PeleError> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut quote = false;
    let mut escaped = false;
    let mut depth = 0usize;
    let mut line = 1usize;
    let mut column = 1usize;
    let mut start_line = 1usize;
    let mut start_column = 1usize;
    let mut has_content = false;
    let mut chars = source.chars().peekable();
    while let Some(ch) = chars.next() {
        if !quote && ch == '%' && chars.peek() == Some(&'%') {
            chars.next();
            for ch in chars.by_ref() {
                if ch == '\n' {
                    if depth == 0 && !current.trim().is_empty() {
                        result.push(Statement {
                            text: current.trim().to_owned(),
                            line: start_line,
                            column: start_column,
                        });
                        current.clear();
                        has_content = false;
                    } else if depth > 0 {
                        current.push(ch);
                    }
                    line += 1;
                    column = 1;
                    break;
                }
            }
            continue;
        }
        if !has_content && !ch.is_whitespace() {
            start_line = line;
            start_column = column;
            has_content = true;
        }
        if ch == '"' && !escaped {
            quote = !quote;
        }
        if !quote {
            match ch {
                '[' | '(' | '{' => depth += 1,
                ']' | ')' | '}' => depth = depth.saturating_sub(1),
                '\n' | ';' if depth == 0 => {
                    if !current.trim().is_empty() {
                        result.push(Statement {
                            text: current.trim().to_owned(),
                            line: start_line,
                            column: start_column,
                        });
                    }
                    current.clear();
                    has_content = false;
                    if ch == '\n' {
                        line += 1;
                        column = 1;
                    } else {
                        column += 1;
                    }
                    escaped = false;
                    continue;
                }
                _ => {}
            }
        }
        current.push(ch);
        escaped = ch == '\\' && !escaped;
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += ch.len_utf16();
        }
    }
    if quote || depth != 0 {
        return Err(PeleError::syntax(
            "Unterminated flowchart label or shape",
            start_line,
            start_column,
        ));
    }
    if !current.trim().is_empty() {
        result.push(Statement {
            text: current.trim().to_owned(),
            line: start_line,
            column: start_column,
        });
    }
    Ok(result)
}

fn keyword<'a>(source: &'a str, word: &str) -> Option<&'a str> {
    source
        .strip_prefix(word)
        .filter(|rest| rest.is_empty() || rest.starts_with(char::is_whitespace))
        .map(str::trim)
}

fn unquote(value: &str) -> String {
    let value = value.trim();
    if value.len() >= 2 && value.starts_with('"') && value.ends_with('"') {
        value[1..value.len() - 1].replace("\\\"", "\"")
    } else {
        value.to_owned()
    }
}

fn split_values(source: &str, delimiter: char) -> Vec<String> {
    let mut values = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    let mut escaped = false;
    let mut depth = 0usize;
    for ch in source.chars() {
        if matches!(ch, '"' | '\'') && !escaped {
            if quote == Some(ch) {
                quote = None;
            } else if quote.is_none() {
                quote = Some(ch);
            }
        }
        if quote.is_none() {
            match ch {
                '(' | '[' | '{' => depth += 1,
                ')' | ']' | '}' => depth = depth.saturating_sub(1),
                ch if ch == delimiter && depth == 0 && !escaped => {
                    values.push(current.trim().to_owned());
                    current.clear();
                    continue;
                }
                _ => {}
            }
        }
        if !escaped || ch != delimiter {
            current.push(ch);
        } else {
            current.pop();
            current.push(ch);
        }
        escaped = ch == '\\' && !escaped;
    }
    if !current.trim().is_empty() {
        values.push(current.trim().to_owned());
    }
    values
}

fn styles(source: &str) -> Result<Vec<String>, PeleError> {
    let mut result = Vec::new();
    for item in split_values(source, ',') {
        for declaration in item.split(';') {
            let Some((property, value)) = declaration.split_once(':') else {
                continue;
            };
            let property = property.trim().to_lowercase();
            let value = value.trim().replace("!important", "");
            let lower = value.to_lowercase();
            if lower.contains("var(") || lower.contains("calc(") || lower.contains("color-mix(") {
                return Err(PeleError::unsupported(
                    "Dynamic CSS expressions in diagram styles are not yet ported",
                ));
            }
            if lower.contains("url(")
                || lower.contains("javascript:")
                || lower.contains("expression(")
                || value.contains(['<', '>', '{', '}', '\\', '@'])
                || lower.contains("/*")
            {
                continue;
            }
            if matches!(
                property.as_str(),
                "fill"
                    | "fill-opacity"
                    | "stroke"
                    | "stroke-width"
                    | "stroke-dasharray"
                    | "stroke-dashoffset"
                    | "stroke-linecap"
                    | "stroke-linejoin"
                    | "stroke-opacity"
                    | "opacity"
                    | "rx"
                    | "ry"
                    | "color"
                    | "font-size"
                    | "font-family"
                    | "font-weight"
                    | "font-style"
                    | "text-decoration"
                    | "letter-spacing"
                    | "word-spacing"
            ) {
                result.push(format!("{property}:{}", value.trim()));
            }
        }
    }
    Ok(result)
}

fn safe_href(value: &str) -> Option<String> {
    let original = text::decode_entities(value).trim().to_owned();
    if original.is_empty() {
        return None;
    }
    let mut decoded = original.clone();
    for _ in 0..32 {
        if decoded.starts_with("//")
            || decoded.contains('\\')
            || decoded
                .chars()
                .any(|ch| ch.is_control() || matches!(ch, '\u{2000}'..='\u{200f}' | '\u{feff}'))
        {
            return None;
        }
        if let Some(colon) = decoded.find(':') {
            let prefix = &decoded[..colon];
            if !prefix.contains(['/', '?', '#'])
                && !matches!(
                    prefix.to_ascii_lowercase().as_str(),
                    "http" | "https" | "mailto" | "tel"
                )
            {
                return None;
            }
        }
        let bytes = decoded.as_bytes();
        let mut unwrapped = Vec::with_capacity(bytes.len());
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'%' && i + 2 < bytes.len() {
                let high = (bytes[i + 1] as char).to_digit(16);
                let low = (bytes[i + 2] as char).to_digit(16);
                if let (Some(high), Some(low)) = (high, low) {
                    unwrapped.push((high * 16 + low) as u8);
                    i += 3;
                    continue;
                }
            }
            unwrapped.push(bytes[i]);
            i += 1;
        }
        let unwrapped = text::decode_entities(std::str::from_utf8(&unwrapped).ok()?);
        if unwrapped == decoded {
            return Some(original);
        }
        decoded = unwrapped;
    }
    None
}

struct Cursor<'a> {
    text: &'a str,
    pos: usize,
    line: usize,
    column: usize,
}

type ParsedLink = (String, Stroke, Marker, Marker, usize, Option<String>);

impl<'a> Cursor<'a> {
    fn rest(&self) -> &'a str {
        &self.text[self.pos..]
    }

    fn space(&mut self) {
        self.pos += self.rest().len() - self.rest().trim_start().len();
    }

    fn error(&self, message: impl Into<String>) -> PeleError {
        let prefix = &self.text[..self.pos];
        let newlines = prefix.chars().filter(|ch| *ch == '\n').count();
        let column = if newlines == 0 {
            self.column + prefix.encode_utf16().count()
        } else {
            prefix
                .rsplit('\n')
                .next()
                .unwrap_or("")
                .encode_utf16()
                .count()
                + 1
        };
        PeleError::syntax(message, self.line + newlines, column)
    }

    fn id(&mut self) -> Result<String, PeleError> {
        self.space();
        let start = self.pos;
        while let Some(ch) = self.rest().chars().next() {
            let rest = self.rest();
            if ch.is_whitespace()
                || "[](){}|&;@<>=\"".contains(ch)
                || rest.starts_with(":::")
                || rest.starts_with("--")
                || rest.starts_with("-.")
                || rest.starts_with("==")
                || rest.starts_with("~~")
            {
                break;
            }
            self.pos += ch.len_utf8();
        }
        if self.pos == start {
            return Err(self.error("Expected a flowchart node identifier"));
        }
        let id = self.text[start..self.pos].to_owned();
        if matches!(id.as_str(), "end" | "subgraph") {
            return Err(self.error("Reserved flowchart keyword used as a node identifier"));
        }
        Ok(id)
    }

    fn delimited(&mut self, open: &str, close: &str) -> Result<String, PeleError> {
        self.pos += open.len();
        let start = self.pos;
        let mut quoted = false;
        let mut escaped = false;
        while !self.rest().is_empty() {
            if !quoted && self.rest().starts_with(close) {
                let value = self.text[start..self.pos].to_owned();
                self.pos += close.len();
                return Ok(unquote(&value));
            }
            let ch = self.rest().chars().next().unwrap();
            if ch == '"' && !escaped {
                quoted = !quoted;
            }
            escaped = ch == '\\' && !escaped;
            self.pos += ch.len_utf8();
        }
        Err(self.error(format!("Expected closing delimiter {close}")))
    }

    fn vertex(
        &mut self,
        model: &mut Model,
        indices: &mut HashMap<String, usize>,
        parent: Option<usize>,
    ) -> Result<usize, PeleError> {
        let id = self.id()?;
        let mut label = None;
        let mut shape = None;
        let delimiters = [
            ("(((", ")))", Shape::DoubleCircle),
            ("((", "))", Shape::Circle),
            ("(-", "-)", Shape::Ellipse),
            ("([", "])", Shape::Stadium),
            ("[[", "]]", Shape::Subroutine),
            ("[(", ")]", Shape::Cylinder),
            ("{{", "}}", Shape::Hexagon),
            ("[", "]", Shape::Rect),
            ("(", ")", Shape::Rounded),
            ("{", "}", Shape::Diamond),
            (">", "]", Shape::Asymmetric),
        ];
        if self.rest().starts_with("[/") || self.rest().starts_with("[\\") {
            let right = self.rest().starts_with("[/");
            self.pos += 2;
            let start = self.pos;
            let mut found = None;
            while self.pos < self.text.len() {
                if self.rest().starts_with("/]") || self.rest().starts_with("\\]") {
                    let close_right = self.rest().starts_with("/]");
                    label = Some(unquote(&self.text[start..self.pos]));
                    shape = Some(match (right, close_right) {
                        (true, true) => Shape::LeanRight,
                        (false, false) => Shape::LeanLeft,
                        (true, false) => Shape::TrapBottom,
                        (false, true) => Shape::TrapTop,
                    });
                    self.pos += 2;
                    found = Some(());
                    break;
                }
                self.pos += self.rest().chars().next().unwrap().len_utf8();
            }
            if found.is_none() {
                return Err(self.error("Unterminated sloped flowchart shape"));
            }
        } else {
            for (open, close, kind) in delimiters {
                if self.rest().starts_with(open) {
                    label = Some(self.delimited(open, close)?);
                    shape = Some(kind);
                    break;
                }
            }
        }
        if self.rest().starts_with("@{") {
            let data = self.delimited("@{", "}")?;
            for field in split_values(&data, ',') {
                let Some((name, value)) = field.split_once(':') else {
                    return Err(self.error("Expected shape metadata key:value"));
                };
                let value = unquote(value.trim().trim_matches('\''));
                match name.trim() {
                    "shape" => {
                        shape = Some(Shape::named(&value).ok_or_else(|| {
                            PeleError::unsupported(format!(
                                "Flowchart shape {value} is not yet ported"
                            ))
                        })?)
                    }
                    "label" => label = Some(value),
                    _ => {
                        return Err(PeleError::unsupported(format!(
                            "Flowchart metadata {} is not yet ported",
                            name.trim()
                        )));
                    }
                }
            }
        }
        let index = if let Some(&index) = indices.get(&id) {
            index
        } else {
            let index = model.nodes.len();
            model.nodes.push(Node {
                label: id.clone(),
                id: id.clone(),
                shape: Shape::Rect,
                styles: Vec::new(),
                classes: Vec::new(),
                parent,
                link: None,
                target: None,
                tooltip: None,
            });
            indices.insert(id, index);
            index
        };
        let node = &mut model.nodes[index];
        if let Some(label) = label {
            node.label = label;
        }
        if let Some(shape) = shape {
            node.shape = shape;
        }
        if node.parent.is_none() {
            node.parent = parent;
        }
        if self.rest().starts_with(":::") {
            self.pos += 3;
            let class = self.id()?;
            node.classes.extend(class.split(',').map(str::to_owned));
        }
        Ok(index)
    }

    fn node_set(
        &mut self,
        model: &mut Model,
        indices: &mut HashMap<String, usize>,
        parent: Option<usize>,
    ) -> Result<Vec<usize>, PeleError> {
        let mut nodes = vec![self.vertex(model, indices, parent)?];
        loop {
            self.space();
            if !self.rest().starts_with('&') {
                break;
            }
            self.pos += 1;
            nodes.push(self.vertex(model, indices, parent)?);
        }
        Ok(nodes)
    }

    fn link(&mut self) -> Result<ParsedLink, PeleError> {
        self.space();
        let mut id = None;
        if let Some(at) = self.rest().find('@') {
            let before = &self.rest()[..at];
            if !before.is_empty()
                && before.chars().all(|ch| ch.is_alphanumeric() || ch == '_')
                && self.rest()[at + 1..].starts_with(['-', '=', '~', '<'])
            {
                id = Some(before.to_owned());
                self.pos += at + 1;
            }
        }
        let mut start = Marker::None;
        if self.rest().starts_with('<') {
            start = Marker::Point;
            self.pos += 1;
        } else if (self.rest().starts_with('o') || self.rest().starts_with('x'))
            && self.rest().get(1..).is_some_and(|s| s.starts_with("--"))
        {
            start = if self.rest().starts_with('o') {
                Marker::Circle
            } else {
                Marker::Cross
            };
            self.pos += 1;
        }
        let begin = self.pos;
        while let Some(ch) = self.rest().chars().next() {
            if !matches!(ch, '-' | '=' | '.' | '~') {
                break;
            }
            self.pos += ch.len_utf8();
        }
        if self.pos == begin {
            return Err(self.error("Expected a flowchart edge"));
        }
        let first = &self.text[begin..self.pos];
        let mut end = Marker::None;
        if let Some(ch) = self.rest().chars().next() {
            end = match ch {
                '>' => Marker::Point,
                'o' => Marker::Circle,
                'x' => Marker::Cross,
                _ => Marker::None,
            };
            if end != Marker::None {
                self.pos += 1;
            }
        }
        let mut source_link = first.to_owned();
        let label;
        if self.rest().trim_start().starts_with('|') {
            self.space();
            label = self.delimited("|", "|")?;
        } else if end == Marker::None && matches!(first, "--" | "-." | "==") {
            self.space();
            let label_start = self.pos;
            let mut quote = false;
            loop {
                if self.rest().is_empty() {
                    return Err(self.error("Expected the end of a labeled edge"));
                }
                if !quote
                    && (self.rest().starts_with("--")
                        || self.rest().starts_with("-.")
                        || self.rest().starts_with(".-")
                        || self.rest().starts_with("=="))
                {
                    label = unquote(&self.text[label_start..self.pos]);
                    let tail_begin = self.pos;
                    while self.rest().starts_with(['-', '.', '=']) {
                        self.pos += 1;
                    }
                    source_link = self.text[tail_begin..self.pos].to_owned();
                    if let Some(ch) = self.rest().chars().next() {
                        end = match ch {
                            '>' => Marker::Point,
                            'o' => Marker::Circle,
                            'x' => Marker::Cross,
                            _ => Marker::None,
                        };
                        if end != Marker::None {
                            self.pos += 1;
                        }
                    }
                    break;
                }
                let ch = self.rest().chars().next().unwrap();
                if ch == '"' {
                    quote = !quote;
                }
                self.pos += ch.len_utf8();
            }
        } else {
            label = String::new();
        }
        if source_link.len() < 2 {
            return Err(self.error("Invalid flowchart edge"));
        }
        let stroke = if source_link.contains('~') {
            Stroke::Invisible
        } else if source_link.contains('=') {
            Stroke::Thick
        } else if source_link.contains('.') {
            Stroke::Dotted
        } else {
            Stroke::Normal
        };
        let count = match stroke {
            Stroke::Dotted => source_link.chars().filter(|ch| *ch == '.').count(),
            Stroke::Thick => source_link
                .chars()
                .filter(|ch| *ch == '=')
                .count()
                .saturating_sub(if end == Marker::None { 2 } else { 1 }),
            _ => source_link
                .chars()
                .filter(|ch| *ch == '-' || *ch == '~')
                .count()
                .saturating_sub(if end == Marker::None { 2 } else { 1 }),
        };
        Ok((label, stroke, start, end, count.clamp(1, 10), id))
    }
}

fn parse(source: &str, max_edges: usize) -> Result<Model, PeleError> {
    let statements = statements(source)?;
    let Some(header) = statements.first() else {
        return Err(PeleError::syntax("Expected a flowchart declaration", 1, 1));
    };
    let header_text = keyword(&header.text, "flowchart")
        .or_else(|| keyword(&header.text, "graph"))
        .ok_or_else(|| {
            PeleError::syntax("Expected flowchart or graph", header.line, header.column)
        })?;
    let direction = if header_text.is_empty() {
        Direction::Tb
    } else {
        Direction::parse(header_text).ok_or_else(|| {
            PeleError::syntax("Expected TB, TD, BT, LR or RL", header.line, header.column)
        })?
    };
    let mut model = Model {
        direction,
        ..Model::default()
    };
    let mut indices = HashMap::<String, usize>::new();
    let mut groups: Vec<usize> = Vec::new();
    let mut group_indices = HashMap::<String, usize>::new();
    let mut pair_count = HashMap::<(usize, usize), usize>::new();
    for statement in statements.iter().skip(1) {
        let line = statement.text.as_str();
        if let Some(rest) = keyword(line, "subgraph") {
            let (id, title) = if rest.is_empty() {
                (format!("subGraph{}", model.groups.len()), String::new())
            } else if let Some(open) = rest.find('[') {
                if !rest.ends_with(']') {
                    return Err(PeleError::syntax(
                        "Unterminated subgraph title",
                        statement.line,
                        statement.column,
                    ));
                }
                let id = rest[..open].trim();
                let id = if id.is_empty() {
                    format!("subGraph{}", model.groups.len())
                } else {
                    id.to_owned()
                };
                (id, unquote(&rest[open + 1..rest.len() - 1]))
            } else {
                let title = unquote(rest);
                let id = if rest.chars().any(char::is_whitespace) || rest.starts_with('"') {
                    format!("subGraph{}", model.groups.len())
                } else {
                    rest.to_owned()
                };
                (id, title)
            };
            let index = if let Some(&index) = group_indices.get(&id) {
                if groups.contains(&index) {
                    return Err(PeleError::syntax(
                        "A subgraph cannot contain itself",
                        statement.line,
                        statement.column,
                    ));
                }
                index
            } else {
                let index = model.groups.len();
                model.groups.push(Group {
                    id: id.clone(),
                    title,
                    parent: groups.last().copied(),
                    direction: None,
                    styles: Vec::new(),
                    classes: Vec::new(),
                });
                group_indices.insert(id, index);
                index
            };
            groups.push(index);
            continue;
        }
        if line == "end" {
            if groups.pop().is_none() {
                return Err(PeleError::syntax(
                    "Unexpected subgraph end",
                    statement.line,
                    statement.column,
                ));
            }
            continue;
        }
        if let Some(rest) = keyword(line, "direction") {
            let direction = Direction::parse(rest).ok_or_else(|| {
                PeleError::syntax("Invalid direction", statement.line, statement.column)
            })?;
            if let Some(&parent) = groups.last() {
                model.groups[parent].direction = Some(direction);
            } else {
                model.direction = direction;
            }
            continue;
        }
        if let Some(rest) = line.strip_prefix("accTitle:") {
            model.acc_title = Some(rest.trim().to_owned());
            continue;
        }
        if let Some(rest) = line.strip_prefix("accDescr:") {
            model.acc_descr = Some(rest.trim().to_owned());
            continue;
        }
        if let Some(rest) = keyword(line, "classDef") {
            let Some((names, declarations)) = rest.split_once(char::is_whitespace) else {
                return Err(PeleError::syntax(
                    "Expected class names and styles",
                    statement.line,
                    statement.column,
                ));
            };
            let declarations = styles(declarations.trim())?;
            for name in names.split(',') {
                model
                    .classes
                    .entry(name.to_owned())
                    .or_default()
                    .extend(declarations.clone());
            }
            continue;
        }
        if let Some(rest) = keyword(line, "class") {
            let Some((ids, classes)) = rest.split_once(char::is_whitespace) else {
                return Err(PeleError::syntax(
                    "Expected identifiers and class names",
                    statement.line,
                    statement.column,
                ));
            };
            for id in ids.split(',') {
                let names = classes.trim().split(',').map(str::to_owned);
                if let Some(&index) = indices.get(id) {
                    model.nodes[index].classes.extend(names);
                } else if let Some(&index) = group_indices.get(id) {
                    model.groups[index].classes.extend(names);
                } else if let Some(edge) = model.edges.iter_mut().find(|edge| edge.id == id) {
                    edge.classes.extend(names);
                }
            }
            continue;
        }
        if let Some(rest) = keyword(line, "style") {
            let Some((id, declarations)) = rest.split_once(char::is_whitespace) else {
                return Err(PeleError::syntax(
                    "Expected an identifier and styles",
                    statement.line,
                    statement.column,
                ));
            };
            let declarations = styles(declarations.trim())?;
            if let Some(&index) = group_indices.get(id) {
                model.groups[index].styles.extend(declarations);
            } else {
                let index = if let Some(&index) = indices.get(id) {
                    index
                } else {
                    let index = model.nodes.len();
                    model.nodes.push(Node {
                        id: id.to_owned(),
                        label: id.to_owned(),
                        shape: Shape::Rect,
                        styles: Vec::new(),
                        classes: Vec::new(),
                        parent: groups.last().copied(),
                        link: None,
                        target: None,
                        tooltip: None,
                    });
                    indices.insert(id.to_owned(), index);
                    index
                };
                model.nodes[index].styles.extend(declarations);
            }
            continue;
        }
        if let Some(rest) = keyword(line, "linkStyle") {
            let Some((ids, declarations)) = rest.split_once(char::is_whitespace) else {
                return Err(PeleError::syntax(
                    "Expected edge indices and styles",
                    statement.line,
                    statement.column,
                ));
            };
            if declarations.trim().starts_with("interpolate") {
                return Err(PeleError::unsupported(
                    "Flowchart edge interpolation directives are not yet ported",
                ));
            }
            let declarations = styles(declarations.trim())?;
            for id in ids.split(',') {
                if id == "default" {
                    model.default_edge_style.extend(declarations.clone());
                } else {
                    let index = id.parse::<usize>().map_err(|_| {
                        PeleError::syntax(
                            "Expected an edge index",
                            statement.line,
                            statement.column,
                        )
                    })?;
                    let Some(edge) = model.edges.get_mut(index) else {
                        return Err(PeleError::syntax(
                            format!("Edge index {index} is out of bounds"),
                            statement.line,
                            statement.column,
                        ));
                    };
                    edge.styles.extend(declarations.clone());
                }
            }
            continue;
        }
        if let Some(rest) = keyword(line, "click") {
            let mut cursor = Cursor {
                text: rest,
                pos: 0,
                line: statement.line,
                column: statement.column + 6,
            };
            let id = cursor.id()?;
            cursor.space();
            if keyword(cursor.rest(), "href").is_some() {
                cursor.pos += 4;
                cursor.space();
            }
            let link = if cursor.rest().starts_with('"') {
                Some(cursor.delimited("\"", "\"")?)
            } else {
                if cursor.rest().is_empty() {
                    return Err(cursor.error("Expected a click callback or URL"));
                }
                if let Some(at) = cursor.rest().find('"') {
                    cursor.pos += at;
                } else {
                    cursor.pos = rest.len();
                }
                None
            };
            cursor.space();
            let tooltip = if cursor.rest().starts_with('"') {
                Some(cursor.delimited("\"", "\"")?)
            } else {
                None
            };
            cursor.space();
            let target = if cursor.rest().is_empty() {
                None
            } else if matches!(cursor.rest(), "_blank" | "_self" | "_parent" | "_top") {
                Some(cursor.rest().to_owned())
            } else {
                return Err(cursor.error("Invalid link target"));
            };
            if let Some(&index) = indices.get(&id) {
                let node = &mut model.nodes[index];
                node.link = link.and_then(|href| safe_href(&href));
                node.target = target;
                node.tooltip = tooltip;
                node.classes.push("clickable".to_owned());
            }
            continue;
        }
        if let Some(at) = line.find("@{") {
            let id = line[..at].trim();
            if model.edges.iter().any(|edge| edge.id == id) {
                let Some(data) = line[at + 2..].strip_suffix('}') else {
                    return Err(PeleError::syntax(
                        "Unterminated edge metadata",
                        statement.line,
                        statement.column + at + 1,
                    ));
                };
                for field in split_values(data, ',') {
                    let Some((name, value)) = field.split_once(':') else {
                        return Err(PeleError::syntax(
                            "Expected edge metadata key:value",
                            statement.line,
                            statement.column + at + 1,
                        ));
                    };
                    match name.trim() {
                        "animate" | "animation" => {}
                        "curve" if unquote(value) == "linear" => {}
                        name => {
                            return Err(PeleError::unsupported(format!(
                                "Flowchart edge metadata {name} is not yet ported"
                            )));
                        }
                    }
                }
                continue;
            }
        }
        let mut cursor = Cursor {
            text: line,
            pos: 0,
            line: statement.line,
            column: statement.column,
        };
        let mut tails = cursor.node_set(&mut model, &mut indices, groups.last().copied())?;
        loop {
            cursor.space();
            if cursor.rest().is_empty() {
                break;
            }
            let (label, stroke, start, end, minlen, id) = cursor.link()?;
            let heads = cursor.node_set(&mut model, &mut indices, groups.last().copied())?;
            let extra = tails
                .len()
                .checked_mul(heads.len())
                .ok_or_else(|| PeleError::limit("Flowchart edge limit exceeded"))?;
            if extra > max_edges.saturating_sub(model.edges.len()) {
                return Err(PeleError::limit(format!(
                    "Edge limit exceeded. The limit is {max_edges}."
                )));
            }
            for &tail in &tails {
                for &head in &heads {
                    let count = pair_count.entry((tail, head)).or_default();
                    let id = id.clone().unwrap_or_else(|| {
                        format!(
                            "L_{}_{}_{}",
                            model.nodes[tail].id,
                            model.nodes[head].id,
                            if *count == 0 { 0 } else { *count + 1 }
                        )
                    });
                    *count += 1;
                    model.edges.push(Edge {
                        id,
                        tail,
                        head,
                        label: label.clone(),
                        stroke,
                        start,
                        end,
                        minlen,
                        styles: Vec::new(),
                        classes: Vec::new(),
                    });
                }
            }
            tails = heads;
        }
    }
    if !groups.is_empty() {
        return Err(PeleError::syntax(
            "Expected subgraph end",
            source.lines().count().max(1),
            1,
        ));
    }
    if model
        .nodes
        .iter()
        .any(|node| group_indices.contains_key(&node.id))
    {
        return Err(PeleError::unsupported(
            "Edges to subgraph identifiers are not yet ported",
        ));
    }
    if model
        .groups
        .iter()
        .any(|group| group.direction.is_some_and(|dir| dir != model.direction))
    {
        return Err(PeleError::unsupported(
            "Independent subgraph directions require the compound layout port",
        ));
    }
    Ok(model)
}

fn compiled_styles(model: &Model, classes: &[String], own: &[String]) -> Vec<String> {
    let mut result = Vec::new();
    for name in classes {
        if let Some(styles) = model.classes.get(name) {
            result.extend(styles.clone());
        }
    }
    result.extend_from_slice(own);
    result
}

fn style_property<'a>(styles: &'a [String], name: &str) -> Option<&'a str> {
    styles.iter().rev().find_map(|item| {
        item.split_once(':')
            .filter(|(property, _)| *property == name)
            .map(|(_, value)| value)
    })
}

fn style_attrs(styles: &[String], text: bool) -> String {
    let declarations: Vec<_> = styles
        .iter()
        .filter_map(|item| {
            let (property, value) = item.split_once(':')?;
            if text {
                match property {
                    "color" => Some(format!("fill:{value}")),
                    "font-size" | "font-family" | "font-weight" | "font-style"
                    | "text-decoration" | "letter-spacing" | "word-spacing" | "opacity" => {
                        Some(item.clone())
                    }
                    _ => None,
                }
            } else if matches!(
                property,
                "fill"
                    | "fill-opacity"
                    | "stroke"
                    | "stroke-width"
                    | "stroke-dasharray"
                    | "stroke-dashoffset"
                    | "stroke-linecap"
                    | "stroke-linejoin"
                    | "stroke-opacity"
                    | "opacity"
                    | "rx"
                    | "ry"
            ) {
                Some(item.clone())
            } else {
                None
            }
        })
        .collect();
    if declarations.is_empty() {
        String::new()
    } else {
        format!(" style=\"{}\"", svg::esc(&declarations.join(";")))
    }
}

fn shape_svg(shape: Shape, w: f64, h: f64, attrs: &str, line_attrs: &str, radius: f64) -> String {
    let n = svg::num;
    let rect = |rx: f64| {
        format!(
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\"{attrs}/>",
            n(-w / 2.0),
            n(-h / 2.0),
            n(w),
            n(h),
            n(rx)
        )
    };
    let polygon = |points: &[(f64, f64)]| {
        format!(
            "<polygon points=\"{}\"{attrs}/>",
            points
                .iter()
                .map(|(x, y)| format!("{},{}", n(*x), n(*y)))
                .collect::<Vec<_>>()
                .join(" ")
        )
    };
    match shape {
        Shape::Rect => rect(radius),
        Shape::Rounded => rect(12.0_f64.min(h / 2.0)),
        Shape::Stadium => rect(h / 2.0),
        Shape::Subroutine => {
            let x = w / 2.0 - 8.0;
            format!(
                "{}<path d=\"M{},{}V{}M{},{}V{}\" fill=\"none\"{line_attrs}/>",
                rect(radius),
                n(-x),
                n(-h / 2.0),
                n(h / 2.0),
                n(x),
                n(-h / 2.0),
                n(h / 2.0)
            )
        }
        Shape::Circle => format!("<circle r=\"{}\"{attrs}/>", n(w / 2.0)),
        Shape::DoubleCircle => format!(
            "<circle r=\"{}\"{attrs}/><circle r=\"{}\" fill=\"none\"{line_attrs}/>",
            n(w / 2.0),
            n(w / 2.0 - 5.0)
        ),
        Shape::Ellipse => format!(
            "<ellipse rx=\"{}\" ry=\"{}\"{attrs}/>",
            n(w / 2.0),
            n(h / 2.0)
        ),
        Shape::Diamond => polygon(&[
            (0.0, -h / 2.0),
            (w / 2.0, 0.0),
            (0.0, h / 2.0),
            (-w / 2.0, 0.0),
        ]),
        Shape::Hexagon => {
            let m = h / 4.0;
            polygon(&[
                (-w / 2.0 + m, -h / 2.0),
                (w / 2.0 - m, -h / 2.0),
                (w / 2.0, 0.0),
                (w / 2.0 - m, h / 2.0),
                (-w / 2.0 + m, h / 2.0),
                (-w / 2.0, 0.0),
            ])
        }
        Shape::LeanRight => polygon(&[
            (-w / 2.0 + h / 4.0, -h / 2.0),
            (w / 2.0, -h / 2.0),
            (w / 2.0 - h / 4.0, h / 2.0),
            (-w / 2.0, h / 2.0),
        ]),
        Shape::LeanLeft => polygon(&[
            (-w / 2.0, -h / 2.0),
            (w / 2.0 - h / 4.0, -h / 2.0),
            (w / 2.0, h / 2.0),
            (-w / 2.0 + h / 4.0, h / 2.0),
        ]),
        Shape::TrapBottom => polygon(&[
            (-w / 2.0 + h / 4.0, -h / 2.0),
            (w / 2.0 - h / 4.0, -h / 2.0),
            (w / 2.0, h / 2.0),
            (-w / 2.0, h / 2.0),
        ]),
        Shape::TrapTop => polygon(&[
            (-w / 2.0, -h / 2.0),
            (w / 2.0, -h / 2.0),
            (w / 2.0 - h / 4.0, h / 2.0),
            (-w / 2.0 + h / 4.0, h / 2.0),
        ]),
        Shape::Asymmetric => polygon(&[
            (-w / 2.0, -h / 2.0),
            (w / 2.0, -h / 2.0),
            (w / 2.0, h / 2.0),
            (-w / 2.0, h / 2.0),
            (-w / 2.0 + h / 4.0, 0.0),
        ]),
        Shape::Cylinder => {
            let rx = w / 2.0;
            let ry = (w / 14.0).clamp(6.0, 12.0);
            let top = -h / 2.0 + ry;
            let bottom = h / 2.0 - ry;
            format!(
                "<path d=\"M{},{}A{},{} 0 0 1 {},{}V{}A{},{} 0 0 1 {},{}Z\"{attrs}/><path d=\"M{},{}A{},{} 0 0 0 {},{}\" fill=\"none\"{line_attrs}/>",
                n(-rx),
                n(top),
                n(rx),
                n(ry),
                n(rx),
                n(top),
                n(bottom),
                n(rx),
                n(ry),
                n(-rx),
                n(bottom),
                n(-rx),
                n(top),
                n(rx),
                n(ry),
                n(rx),
                n(top)
            )
        }
        Shape::Text => String::new(),
        Shape::Start => format!("<circle r=\"{}\"{line_attrs}/>", n(w / 2.0)),
        Shape::End => format!(
            "<circle r=\"{}\"{attrs}/><circle r=\"{}\"{line_attrs}/>",
            n(w / 2.0),
            n(w / 2.0 - 4.0)
        ),
        Shape::Fork => rect(2.0),
        Shape::Extended(name) => crate::flow_shapes::draw(name, w, h, attrs, line_attrs, radius)
            .expect("canonical extended shape"),
    }
}

fn marker_svg(marker: Marker, x: f64, y: f64, dx: f64, dy: f64, color: &str) -> String {
    let magnitude = dx.hypot(dy);
    if marker == Marker::None || magnitude < 0.0001 {
        return String::new();
    }
    let ux = dx / magnitude;
    let uy = dy / magnitude;
    let n = svg::num;
    match marker {
        Marker::Point => format!(
            "<path d=\"M{},{}L{},{}L{},{}Z\" fill=\"{}\"/>",
            n(x),
            n(y),
            n(x - ux * 9.0 + uy * 4.0),
            n(y - uy * 9.0 - ux * 4.0),
            n(x - ux * 9.0 - uy * 4.0),
            n(y - uy * 9.0 + ux * 4.0),
            svg::esc(color)
        ),
        Marker::Circle => format!(
            "<circle cx=\"{}\" cy=\"{}\" r=\"4\" fill=\"{}\"/>",
            n(x - ux * 4.0),
            n(y - uy * 4.0),
            svg::esc(color)
        ),
        Marker::Cross => format!(
            "<path d=\"M{},{}L{},{}M{},{}L{},{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\"/>",
            n(x - ux * 3.0 + uy * 4.0),
            n(y - uy * 3.0 - ux * 4.0),
            n(x - ux * 3.0 - uy * 4.0),
            n(y - uy * 3.0 + ux * 4.0),
            n(x - ux * 7.0 + uy * 4.0),
            n(y - uy * 7.0 - ux * 4.0),
            n(x + ux + uy * -4.0),
            n(y + uy + ux * 4.0),
            svg::esc(color)
        ),
        Marker::None => String::new(),
    }
}

pub fn render(source: &str, options: &RenderOptions) -> Result<RenderResult, PeleError> {
    let mut model = parse(source, options.max_edges)?;
    let across = fitted(&model, options)?;
    if model.direction.sideways()
        && options.auto_direction
        && options.max_width.is_some_and(|width| {
            width > 0.0 && width < options.direction_breakpoint && across.width > width
        })
    {
        model.direction = Direction::Tb;
        let down = fitted(&model, options)?;
        if down.width < across.width {
            return Ok(down);
        }
    }
    Ok(across)
}

fn fitted(model: &Model, options: &RenderOptions) -> Result<RenderResult, PeleError> {
    let loose = draw(model, options, false)?;
    if options
        .max_width
        .is_some_and(|width| width > 0.0 && loose.width > width)
    {
        let tight = draw(model, options, true)?;
        if tight.width < loose.width {
            return Ok(tight);
        }
    }
    Ok(loose)
}

fn draw(model: &Model, options: &RenderOptions, tight: bool) -> Result<RenderResult, PeleError> {
    let n = svg::num;
    let draw_label = |label: &text::Label, x: f64, y: f64, attrs: &str| {
        svg::label_with_options(label, x, y, attrs, options)
    };
    let count = model.nodes.len();
    let sideways = model.direction.sideways();
    let measurer = options
        .measurer
        .as_deref()
        .unwrap_or(&text::MetricsMeasurer);
    let flow_config = &options.config["flowchart"];
    let number_option = |key: &str, fallback: f64| {
        flow_config[key]
            .as_f64()
            .filter(|value| value.is_finite() && *value > 0.0)
            .unwrap_or(fallback)
            .min(2000.0)
    };
    let wrap_width = number_option("wrappingWidth", 200.0);
    let make_label = |raw: &str, size: f64, max_width: Option<f64>| {
        text::label_with_measurer(raw, size, max_width, measurer)
    };
    let node_styles: Vec<_> = model
        .nodes
        .iter()
        .map(|node| {
            let mut names = vec!["default".to_owned(), "node".to_owned()];
            names.extend_from_slice(&node.classes);
            compiled_styles(model, &names, &node.styles)
        })
        .collect();
    let labels: Vec<_> = model
        .nodes
        .iter()
        .zip(&node_styles)
        .map(|(node, styles)| {
            let size = style_property(styles, "font-size")
                .and_then(|value| value.trim_end_matches("px").parse::<f64>().ok())
                .filter(|size| *size > 0.0 && *size <= 200.0)
                .unwrap_or(options.font_size);
            if !node.shape.has_label() {
                return text::Label::default();
            }
            let markdown = node.label.starts_with('\u{60}')
                && node.label.ends_with('\u{60}')
                && node.label.len() >= 2;
            let raw = if markdown {
                &node.label[1..node.label.len() - 1]
            } else {
                &node.label
            };
            let mut base = 0;
            if style_property(styles, "font-weight").is_some_and(|weight| {
                weight == "bold"
                    || weight == "bolder"
                    || weight.parse::<u16>().is_ok_and(|weight| weight >= 600)
            }) {
                base |= text::BOLD;
            }
            if style_property(styles, "font-style") == Some("italic") {
                base |= text::ITALIC;
            }
            text::layout_label(raw, markdown, measurer, size, wrap_width, base)
        })
        .collect();
    let dimensions: Vec<_> = model
        .nodes
        .iter()
        .zip(&labels)
        .map(|(node, label)| node.shape.size(label.width, label.height))
        .collect();
    let edge_labels: Vec<_> = model
        .edges
        .iter()
        .map(|edge| {
            make_label(
                &edge.label,
                (options.font_size * 0.875).round(),
                Some(wrap_width),
            )
        })
        .collect();
    let cluster_room = if model.groups.is_empty() { 0.0 } else { 48.0 };
    let mut nodes: Vec<_> = dimensions
        .iter()
        .enumerate()
        .map(|(index, (w, h, _))| {
            let mut node = layout::lnode(
                if sideways { *h } else { *w },
                if sideways { *w } else { *h },
            );
            node.span = model.nodes[index].shape.span(*w, *h, sideways);
            node
        })
        .collect();
    let mut edges: Vec<_> = model
        .edges
        .iter()
        .zip(&edge_labels)
        .map(|(edge, label)| {
            let width = if label.width > 0.0 {
                label.width + 12.0
            } else {
                0.0
            };
            let height = if label.height > 0.0 {
                label.height + 4.0
            } else {
                0.0
            };
            layout::ledge_with(
                edge.tail,
                edge.head,
                edge.minlen,
                if sideways { height } else { width },
                if sideways { width } else { height },
            )
        })
        .collect();
    let groups = if model.groups.is_empty() {
        None
    } else {
        Some(Groups {
            of: model
                .nodes
                .iter()
                .map(|node| node.parent.map_or(-1, |index| index as i32))
                .collect(),
            parent: model
                .groups
                .iter()
                .map(|group| group.parent.map_or(-1, |index| index as i32))
                .collect(),
        })
    };
    let result = layout::layered(
        &mut nodes,
        &mut edges,
        &LayeredOptions {
            node_sep: number_option("nodeSpacing", 40.0) * if tight { 0.5 } else { 1.0 }
                + cluster_room,
            edge_sep: 16.0,
            rank_sep: number_option("rankSpacing", 48.0) * if tight { 0.75 } else { 1.0 }
                + cluster_room,
            port_sep: 16.0,
            groups,
            ..LayeredOptions::default()
        },
    );
    let extra_pad = options.padding
        + if model.groups.is_empty() {
            0.0
        } else {
            28.0 * model.groups.len().min(16) as f64
        };
    let (mut width, mut height) = if sideways {
        (result.height, result.width)
    } else {
        (result.width, result.height)
    };
    let transform = |x: f64, y: f64| -> (f64, f64) {
        let (x, y) = match model.direction {
            Direction::Tb => (x, y),
            Direction::Bt => (x, result.height - y),
            Direction::Lr => (y, x),
            Direction::Rl => (result.height - y, x),
        };
        (x + extra_pad, y + extra_pad)
    };
    width += 2.0 * extra_pad;
    height += 2.0 * extra_pad;
    width = width.max(2.0 * options.padding + 1.0);
    height = height.max(2.0 * options.padding + 1.0);
    let positions: Vec<_> = nodes
        .iter()
        .take(count)
        .map(|node| transform(node.x, node.y))
        .collect();
    let mut contents = String::new();
    let mut links = Vec::new();
    let mut group_boxes = vec![None::<(f64, f64, f64, f64)>; model.groups.len()];
    for (i, node) in model.nodes.iter().enumerate() {
        if let Some(parent) = node.parent {
            let (x, y) = positions[i];
            let (w, h, _) = dimensions[i];
            let value = (x - w / 2.0, y - h / 2.0, x + w / 2.0, y + h / 2.0);
            group_boxes[parent] = Some(if let Some(old) = group_boxes[parent] {
                (
                    old.0.min(value.0),
                    old.1.min(value.1),
                    old.2.max(value.2),
                    old.3.max(value.3),
                )
            } else {
                value
            });
        }
    }
    for i in (0..model.groups.len()).rev() {
        let label = make_label(&model.groups[i].title, options.font_size, Some(4000.0));
        if let Some(old) = group_boxes[i] {
            let top = old.1 - label.height - 16.0;
            let center = (old.0 + old.2) / 2.0;
            let w = (old.2 - old.0 + 40.0).max(label.width + 28.0);
            let value = (center - w / 2.0, top, center + w / 2.0, old.3 + 20.0);
            group_boxes[i] = Some(value);
            if let Some(parent) = model.groups[i].parent {
                group_boxes[parent] = Some(if let Some(old) = group_boxes[parent] {
                    (
                        old.0.min(value.0),
                        old.1.min(value.1),
                        old.2.max(value.2),
                        old.3.max(value.3),
                    )
                } else {
                    value
                });
            }
        }
    }
    for (i, group) in model.groups.iter().enumerate() {
        let Some((left, top, right, bottom)) = group_boxes[i] else {
            continue;
        };
        let styles = compiled_styles(model, &group.classes, &group.styles);
        let label = make_label(&group.title, options.font_size, Some(4000.0));
        let fill = if group.parent.is_none() {
            &options.theme.surface_alt
        } else {
            &options.theme.surface
        };
        contents.push_str(&format!("<g class=\"pele-cluster\" data-id=\"{}\"><rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" fill=\"{}\" stroke=\"{}\"{}/>{}</g>",svg::esc(&group.id),n(left),n(top),n(right-left),n(bottom-top),n(options.theme.radius),svg::esc(fill),svg::esc(&options.theme.border),style_attrs(&styles,false),draw_label(&label,(left+right)/2.0,top+label.height/2.0+4.0,&format!(" class=\"pele-cluster-label\" fill=\"{}\"{}",svg::esc(&options.theme.muted),style_attrs(&styles,true)))));
    }
    for (i, edge) in model.edges.iter().enumerate() {
        if edge.stroke == Stroke::Invisible {
            continue;
        }
        let mut own = model.default_edge_style.clone();
        own.extend_from_slice(&edge.styles);
        let styles = compiled_styles(model, &edge.classes, &own);
        let color = style_property(&styles, "stroke").unwrap_or(&options.theme.line);
        let mut points: Vec<_> = edges[i]
            .points
            .as_chunks::<2>()
            .0
            .iter()
            .map(|point| transform(point[0], point[1]))
            .collect();
        if edge.tail == edge.head {
            let (x, y) = positions[edge.tail];
            let (w, h, _) = dimensions[edge.tail];
            points = if sideways {
                vec![
                    (x - w / 4.0, y - h / 2.0),
                    (x - w / 4.0, y - h / 2.0 - 28.0),
                    (x + w / 4.0, y - h / 2.0 - 28.0),
                    (x + w / 4.0, y - h / 2.0),
                ]
            } else {
                vec![
                    (x + w / 2.0, y - h / 4.0),
                    (x + w / 2.0 + 28.0, y - h / 4.0),
                    (x + w / 2.0 + 28.0, y + h / 4.0),
                    (x + w / 2.0, y + h / 4.0),
                ]
            };
        }
        if points.len() < 2 {
            let (a, b) = (positions[edge.tail], positions[edge.head]);
            points = vec![a, b];
        }
        for (at, index) in [(0, edge.tail), (points.len() - 1, edge.head)] {
            let (cx, cy) = positions[index];
            let (w, h, _) = dimensions[index];
            let (x, y) = points[at];
            let (dx, dy) = (x - cx, y - cy);
            if (dy.abs() - h / 2.0).abs() < 0.5 {
                let side = if dy < 0.0 { 0 } else { 2 };
                let amount = model.nodes[index].shape.inset(w, h, side, dx);
                points[at].1 += if side == 0 { amount } else { -amount };
            } else if (dx.abs() - w / 2.0).abs() < 0.5 {
                let side = if dx < 0.0 { 3 } else { 1 };
                let amount = model.nodes[index].shape.inset(w, h, side, dy);
                points[at].0 += if side == 3 { amount } else { -amount };
            }
        }
        let path = points
            .iter()
            .enumerate()
            .map(|(index, (x, y))| {
                format!("{}{},{}", if index == 0 { "M" } else { "L" }, n(*x), n(*y))
            })
            .collect::<String>();
        let dash = if edge.stroke == Stroke::Dotted {
            " stroke-dasharray=\"4 4\""
        } else {
            ""
        };
        let thickness = if edge.stroke == Stroke::Thick {
            "3"
        } else {
            "1.5"
        };
        contents.push_str(&format!("<g class=\"pele-edge\" data-id=\"{}\"><path d=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{thickness}\" stroke-linejoin=\"round\"{dash}{}/>",svg::esc(&edge.id),path,svg::esc(color),style_attrs(&styles,false)));
        let a = points[0];
        let b = points[1];
        let z = points[points.len() - 1];
        let y = points[points.len() - 2];
        contents.push_str(&marker_svg(
            edge.start,
            a.0,
            a.1,
            a.0 - b.0,
            a.1 - b.1,
            color,
        ));
        contents.push_str(&marker_svg(edge.end, z.0, z.1, z.0 - y.0, z.1 - y.1, color));
        if !edge.label.is_empty() {
            let (x, y) = if edge.tail == edge.head {
                points[points.len() / 2]
            } else {
                transform(edges[i].label_x, edges[i].label_y)
            };
            let label = &edge_labels[i];
            contents.push_str(&format!(
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"2\" fill=\"{}\"/>{}",
                n(x - label.width / 2.0 - 4.0),
                n(y - label.height / 2.0 - 2.0),
                n(label.width + 8.0),
                n(label.height + 4.0),
                svg::esc(&options.theme.background),
                draw_label(
                    label,
                    x,
                    y,
                    &format!(
                        " class=\"pele-edge-label\" fill=\"{}\"{}",
                        svg::esc(&options.theme.foreground),
                        style_attrs(&styles, true)
                    )
                )
            ));
        }
        contents.push_str("</g>");
    }
    for (i, node) in model.nodes.iter().enumerate() {
        let (x, y) = positions[i];
        let (w, h, dy) = dimensions[i];
        let styles = &node_styles[i];
        let attrs = format!(
            " fill=\"{}\" stroke=\"{}\"{}",
            svg::esc(&options.theme.surface),
            svg::esc(&options.theme.border),
            style_attrs(styles, false)
        );
        let line_attrs = format!(
            " stroke=\"{}\"{}",
            svg::esc(&options.theme.border),
            style_attrs(styles, false)
        );
        let shape = if let Shape::Extended(name) = node.shape {
            crate::flow_shapes::draw_with_line_color(
                name,
                w,
                h,
                &attrs,
                &line_attrs,
                options.theme.radius,
                &options.theme.line,
            )
            .expect("canonical extended shape")
        } else {
            shape_svg(node.shape, w, h, &attrs, &line_attrs, options.theme.radius)
        };
        let label = if !node.shape.has_label() {
            String::new()
        } else {
            draw_label(
                &labels[i],
                0.0,
                dy,
                &format!(
                    " class=\"pele-label\" fill=\"{}\"{}",
                    svg::esc(&options.theme.foreground),
                    style_attrs(styles, true)
                ),
            )
        };
        let classes = if node.classes.is_empty() {
            String::new()
        } else {
            format!(" {}", svg::esc(&node.classes.join(" ")))
        };
        let tooltip = node.tooltip.as_ref().map_or(String::new(), |value| {
            format!("<title>{}</title>", svg::esc(value))
        });
        let body = format!(
            "<g class=\"pele-node{classes}\" data-id=\"{}\" transform=\"translate({},{})\">{tooltip}{shape}{label}</g>",
            svg::esc(&node.id),
            n(x),
            n(y)
        );
        if let Some(href) = &node.link {
            let target = node.target.as_ref().map_or(String::new(), |value| {
                format!(" target=\"{}\"", svg::esc(value))
            });
            contents.push_str(&format!(
                "<a href=\"{}\"{target} rel=\"noopener\">{body}</a>",
                svg::esc(href)
            ));
            links.push(crate::LinkInfo {
                id: node.id.clone(),
                href: href.clone(),
                internal: false,
            });
        } else {
            contents.push_str(&body);
        }
    }
    let mut bounds = (0.0_f64, 0.0_f64, width, height);
    for &(left, top, right, bottom) in group_boxes.iter().flatten() {
        bounds.0 = bounds.0.min(left - options.padding);
        bounds.1 = bounds.1.min(top - options.padding);
        bounds.2 = bounds.2.max(right + options.padding);
        bounds.3 = bounds.3.max(bottom + options.padding);
    }
    for edge in &model.edges {
        if edge.tail == edge.head {
            let (x, y) = positions[edge.tail];
            let (w, h, _) = dimensions[edge.tail];
            bounds.1 = bounds.1.min(y - h / 2.0 - 28.0 - options.padding);
            bounds.2 = bounds.2.max(x + w / 2.0 + 28.0 + options.padding);
        }
    }
    width = bounds.2 - bounds.0;
    height = bounds.3 - bounds.1;
    let title_room = if tight {
        options
            .max_width
            .map(|width| (width - 2.0 * options.padding).max(80.0))
            .unwrap_or(4000.0)
    } else {
        4000.0
    };
    let title = options
        .title
        .as_ref()
        .map(|title| make_label(title, options.font_size * 1.1, Some(title_room)));
    let title_height = title.as_ref().map_or(0.0, |label| label.height + 16.0);
    if let Some(label) = &title {
        width = width.max(label.width + 2.0 * options.padding);
    }
    contents = format!(
        "<g transform=\"translate({},{})\">{contents}</g>",
        n(-bounds.0),
        n(-bounds.1 + title_height)
    );
    if let Some(label) = &title {
        contents = format!(
            "{}{}",
            draw_label(
                label,
                width / 2.0,
                options.padding + label.height / 2.0,
                &format!(
                    " class=\"pele-title\" font-weight=\"bold\" fill=\"{}\"",
                    svg::esc(&options.theme.foreground)
                )
            ),
            contents
        );
    }
    height += title_height;
    let mut effective = options.clone();
    effective.acc_title = model.acc_title.clone().or(effective.acc_title);
    effective.acc_descr = model.acc_descr.clone().or(effective.acc_descr);
    width = width.ceil();
    height = height.ceil();
    let svg = svg::document("flowchart", width, height, &effective, &contents);
    let mut result = RenderResult::new("flowchart", svg, width, height);
    result.links = links;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_chain_labels_shapes_and_direction() {
        let model = parse(
            "flowchart LR\n A([Start]) -->|go| B{Choice} --> C[(Database)]",
            10,
        )
        .unwrap();
        assert_eq!(model.direction, Direction::Lr);
        assert_eq!(model.nodes.len(), 3);
        assert_eq!(model.nodes[0].shape, Shape::Stadium);
        assert_eq!(model.nodes[1].shape, Shape::Diamond);
        assert_eq!(model.nodes[2].shape, Shape::Cylinder);
        assert_eq!(model.edges[0].label, "go");
        assert_eq!(model.edges[1].head, 2);
    }

    #[test]
    fn cartesian_edges_are_bounded_before_expansion() {
        let source = "graph TD;A & B --> C & D";
        assert!(parse(source, 3).is_err());
        assert_eq!(parse(source, 4).unwrap().edges.len(), 4);
    }

    #[test]
    fn redefinitions_preserve_insertion_order_and_last_label() {
        let model = parse("graph TD;A[old]-->B;A[new];B-->A", 10).unwrap();
        assert_eq!(model.nodes.len(), 2);
        assert_eq!(model.nodes[0].label, "new");
        assert_eq!(model.edges.len(), 2);
    }

    #[test]
    fn quoted_labels_keep_semicolons_and_newlines() {
        let model = parse("graph TD;A[\"x;y\nz\"]-->B", 10).unwrap();
        assert_eq!(model.nodes[0].label, "x;y\nz");
    }

    #[test]
    fn supports_labeled_link_forms_and_stroke_patterns() {
        let model = parse(
            "graph LR;A -- hello --> B;B -. retry .-> C;C ==> D;D o--o E;E ~~~ F",
            20,
        )
        .unwrap();
        assert_eq!(model.edges[0].label, "hello");
        assert_eq!(model.edges[1].stroke, Stroke::Dotted);
        assert_eq!(model.edges[1].label, "retry");
        assert_eq!(model.edges[2].stroke, Stroke::Thick);
        assert_eq!(model.edges[3].start, Marker::Circle);
        assert_eq!(model.edges[3].end, Marker::Circle);
        assert_eq!(model.edges[4].stroke, Stroke::Invisible);
    }

    #[test]
    fn repeated_subgraphs_merge_and_nesting_is_preserved() {
        let model = parse(
            "graph TD;subgraph S[Outer];subgraph T[Inner];A-->B;end;end;subgraph S;C;end",
            10,
        )
        .unwrap();
        assert_eq!(model.groups.len(), 2);
        assert_eq!(model.groups[1].parent, Some(0));
        assert_eq!(model.nodes[0].parent, Some(1));
        assert_eq!(model.nodes[2].parent, Some(0));
    }

    #[test]
    fn class_and_explicit_styles_are_resolved_in_order() {
        let model = parse(
            "flowchart TD;A:::hot-->B;classDef hot fill:#f00,color:#fff;style A fill:#0f0",
            10,
        )
        .unwrap();
        let styles = compiled_styles(&model, &model.nodes[0].classes, &model.nodes[0].styles);
        assert_eq!(style_property(&styles, "fill"), Some("#0f0"));
        assert_eq!(style_property(&styles, "color"), Some("#fff"));
    }

    #[test]
    fn metadata_selects_shapes_and_quoted_labels() {
        let model = parse("flowchart LR;A@{shape: diamond, label: \"x,y\"}-->B", 10).unwrap();
        assert_eq!(model.nodes[0].shape, Shape::Diamond);
        assert_eq!(model.nodes[0].label, "x,y");
        assert!(parse("flowchart LR;A@{shape: made-up}", 10).is_err());
    }

    #[test]
    fn reports_invalid_syntax_and_unported_features() {
        for source in [
            "flowchart LR;A[broken",
            "flowchart LR;A-->[bad]",
            "flowchart LR;end",
            "flowchart LR;subgraph S;A",
        ] {
            assert!(parse(source, 20).is_err(), "{source}");
        }
        assert!(parse("flowchart LR;A;style A fill:var(--anything)", 20).is_err());
        assert!(parse("flowchart LR;subgraph S;direction TB;A;end", 20).is_err());
    }

    #[test]
    fn static_svg_is_escaped_deterministic_and_oriented() {
        let options = RenderOptions::default();
        let source = "flowchart LR;A[\"<script>&quoted\"]-->B[Done]";
        let a = render(source, &options).unwrap();
        let b = render(source, &options).unwrap();
        assert_eq!(a.svg, b.svg);
        assert!(a.width.is_finite() && a.height.is_finite());
        assert!(a.svg.contains("&amp;quoted"));
        assert!(!a.svg.contains("<script>"));
        assert!(!a.svg.contains("var("));
        assert!(!a.svg.contains("<foreignObject"));
        assert!(a.svg.contains("data-id=\"A\""));
        let generic = render("flowchart LR;A[\"List<T>\"]", &options).unwrap();
        assert!(generic.svg.contains("List&lt;T&gt;"));
    }

    #[test]
    fn links_keep_metadata_and_tooltips_without_executing_callbacks() {
        let options = RenderOptions::default();
        let result=render("flowchart LR;A-->B;click A \"https://example.com?q=A%26B\" \"Go here\" _blank;click B call callback() \"A tooltip\"",&options).unwrap();
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.links[0].href, "https://example.com?q=A%26B");
        assert!(result.svg.contains("target=\"_blank\" rel=\"noopener\""));
        assert!(result.svg.contains("<title>A tooltip</title>"));
        assert!(!result.svg.contains("callback()"));
    }

    #[test]
    fn unsafe_link_schemes_and_obfuscations_are_omitted() {
        for url in [
            "javascript:alert(1)",
            "data:text/html,hello",
            "//example.com",
            "\\\\example.com",
            "%6aavascript%3aalert(1)",
            "javascript&colon;alert(1)",
            "%252f%252fexample.com",
            "file:///tmp/private",
        ] {
            let result = render(
                &format!("flowchart LR;A;click A \"{url}\""),
                &RenderOptions::default(),
            )
            .unwrap();
            assert!(result.links.is_empty(), "{url}");
            assert!(!result.svg.contains("<a href"), "{url}");
        }
    }

    #[test]
    fn explicit_font_measurements_affect_layout() {
        let normal = render("flowchart LR;A[Long label]-->B", &RenderOptions::default()).unwrap();
        let options = RenderOptions {
            measurer: Some(std::sync::Arc::new(|text: &str, size: f64, _style: u8| {
                text.chars().count() as f64 * size * 2.0
            })),
            ..RenderOptions::default()
        };
        let wide = render("flowchart LR;A[Long label]-->B", &options).unwrap();
        assert!(wide.width > normal.width);
    }

    #[test]
    fn narrow_hosts_turn_horizontal_flow_and_honor_opt_out() {
        let source = "flowchart LR;A-->B-->C-->D-->E";
        let options = RenderOptions {
            max_width: Some(300.0),
            ..RenderOptions::default()
        };
        let down = render(source, &options).unwrap();
        assert!(down.width < down.height);
        assert!(down.width <= 300.0);
        let fixed = render(
            source,
            &RenderOptions {
                auto_direction: false,
                ..options.clone()
            },
        )
        .unwrap();
        assert!(fixed.width > fixed.height);
        let below_breakpoint = render(
            source,
            &RenderOptions {
                direction_breakpoint: 250.0,
                ..options
            },
        )
        .unwrap();
        assert!(below_breakpoint.width > below_breakpoint.height);
    }

    #[test]
    fn extended_shape_port_renders_all_named_definitions() {
        for name in crate::flow_shapes::SHAPES {
            let result = render(
                &format!("flowchart TD;A@{{shape: {name}, label: \"Label\"}}"),
                &RenderOptions::default(),
            )
            .unwrap();
            assert!(
                result.width.is_finite() && result.height.is_finite(),
                "{name}"
            );
            assert!(!result.svg.contains("var("), "{name}");
            assert!(!result.svg.contains("currentColor"), "{name}");
        }
    }

    #[test]
    fn numeric_style_fragments_and_headless_edge_metadata_match_upstream() {
        let model=parse("flowchart LR;A e1@--> B;e1@{animate:true};classDef animate stroke-dasharray:9,5,stroke-dashoffset:900,animation:dash 25s linear infinite;class e1 animate",20).unwrap();
        assert_eq!(model.nodes.len(), 2);
        assert_eq!(model.edges.len(), 1);
        assert_eq!(model.edges[0].classes, ["animate"]);
        let styles = compiled_styles(&model, &model.edges[0].classes, &[]);
        assert_eq!(style_property(&styles, "stroke-dasharray"), Some("9"));
        assert_eq!(style_property(&styles, "stroke-dashoffset"), Some("900"));
    }

    #[test]
    fn syntax_columns_use_utf16_offsets() {
        let error = parse("flowchart LR\n A[\"👋\"] --> @", 20).unwrap_err();
        assert_eq!(error.line, 2);
        assert_eq!(error.column, 14);
    }
}
