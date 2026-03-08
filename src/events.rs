use std::collections::BTreeMap;
use std::io::{self, Read};

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::Result;

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MessageFormat {
    #[default]
    Compact,
    Alert,
    Inline,
    Raw,
}

impl MessageFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Compact => "compact",
            Self::Alert => "alert",
            Self::Inline => "inline",
            Self::Raw => "raw",
        }
    }

    pub fn from_label(label: &str) -> Result<Self> {
        match label {
            "compact" => Ok(Self::Compact),
            "alert" => Ok(Self::Alert),
            "inline" => Ok(Self::Inline),
            "raw" => Ok(Self::Raw),
            other => Err(format!("unsupported message format: {other}").into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncomingEvent {
    #[serde(rename = "type", alias = "kind", alias = "event")]
    pub kind: String,
    #[serde(default)]
    pub channel: Option<String>,
    #[serde(default)]
    pub format: Option<MessageFormat>,
    #[serde(default)]
    pub template: Option<String>,
    #[serde(default)]
    pub payload: Value,
}

impl IncomingEvent {
    pub fn custom(channel: Option<String>, message: String) -> Self {
        Self {
            kind: "custom".to_string(),
            channel,
            format: None,
            template: None,
            payload: json!({ "message": message }),
        }
    }

    pub fn github_issue_opened(
        repo: String,
        number: u64,
        title: String,
        channel: Option<String>,
    ) -> Self {
        Self {
            kind: "github.issue-opened".to_string(),
            channel,
            format: None,
            template: None,
            payload: json!({
                "repo": repo,
                "number": number,
                "title": title,
            }),
        }
    }

    pub fn tmux_keyword(
        session: String,
        keyword: String,
        line: String,
        channel: Option<String>,
    ) -> Self {
        Self {
            kind: "tmux.keyword".to_string(),
            channel,
            format: None,
            template: None,
            payload: json!({
                "session": session,
                "keyword": keyword,
                "line": line,
            }),
        }
    }

    pub fn canonical_kind(&self) -> &str {
        match self.kind.as_str() {
            "issue-opened" => "github.issue-opened",
            other => other,
        }
    }

    pub fn render_default(&self, format: &MessageFormat) -> Result<String> {
        let payload = &self.payload;
        let text = match (self.canonical_kind(), format) {
            ("custom", MessageFormat::Compact | MessageFormat::Inline) => {
                string_field(payload, "message")?
            }
            ("custom", MessageFormat::Alert) => format!("🚨 {}", string_field(payload, "message")?),
            ("custom", MessageFormat::Raw) => serde_json::to_string_pretty(payload)?,
            ("github.issue-opened", MessageFormat::Compact) => format!(
                "{}#{} opened: {}",
                string_field(payload, "repo")?,
                payload.field_u64("number")?,
                string_field(payload, "title")?
            ),
            ("github.issue-opened", MessageFormat::Alert) => format!(
                "🚨 GitHub issue opened in {}: #{} {}",
                string_field(payload, "repo")?,
                payload.field_u64("number")?,
                string_field(payload, "title")?
            ),
            ("github.issue-opened", MessageFormat::Inline) => format!(
                "[GitHub] {}#{} {}",
                string_field(payload, "repo")?,
                payload.field_u64("number")?,
                string_field(payload, "title")?
            ),
            ("github.issue-opened", MessageFormat::Raw) => serde_json::to_string_pretty(payload)?,
            ("tmux.keyword", MessageFormat::Compact) => format!(
                "tmux:{} matched '{}' => {}",
                string_field(payload, "session")?,
                string_field(payload, "keyword")?,
                string_field(payload, "line")?
            ),
            ("tmux.keyword", MessageFormat::Alert) => format!(
                "🚨 tmux session {} hit keyword '{}': {}",
                string_field(payload, "session")?,
                string_field(payload, "keyword")?,
                string_field(payload, "line")?
            ),
            ("tmux.keyword", MessageFormat::Inline) => format!(
                "[tmux:{}] {}",
                string_field(payload, "session")?,
                string_field(payload, "line")?
            ),
            (_, MessageFormat::Raw) => serde_json::to_string_pretty(payload)?,
            (_, _) => serde_json::to_string(payload)?,
        };
        Ok(text)
    }

    pub fn template_context(&self) -> BTreeMap<String, String> {
        let mut context = BTreeMap::new();
        context.insert("kind".to_string(), self.canonical_kind().to_string());
        flatten_json("", &self.payload, &mut context);
        context
    }
}

pub fn render_template(template: &str, context: &BTreeMap<String, String>) -> String {
    let mut rendered = template.to_string();
    for (key, value) in context {
        let pattern = format!("{{{key}}}");
        rendered = rendered.replace(&pattern, value);
    }
    rendered
}

pub fn read_stdin() -> Result<String> {
    let mut buf = String::new();
    io::stdin().read_to_string(&mut buf)?;
    Ok(buf)
}

pub fn parse_stream(body: &str) -> Result<Vec<IncomingEvent>> {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    if trimmed.starts_with('[') {
        return serde_json::from_str::<Vec<IncomingEvent>>(trimmed)
            .map(|events| events.into_iter().map(normalize_event).collect())
            .map_err(Into::into);
    }

    if !trimmed.contains('\n') && trimmed.starts_with('{') {
        return serde_json::from_str::<IncomingEvent>(trimmed)
            .map(|event| vec![normalize_event(event)])
            .map_err(Into::into);
    }

    trimmed
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<IncomingEvent>(line).map(normalize_event))
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(Into::into)
}

pub fn normalize_event(mut event: IncomingEvent) -> IncomingEvent {
    if !event.payload.is_object() {
        event.payload = json!({ "value": event.payload });
    }
    event.kind = event.canonical_kind().to_string();
    event
}

fn string_field(payload: &Value, key: &str) -> Result<String> {
    payload
        .get(key)
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .ok_or_else(|| format!("missing string field '{key}'").into())
}

fn flatten_json(prefix: &str, value: &Value, out: &mut BTreeMap<String, String>) {
    match value {
        Value::Object(map) => {
            for (key, value) in map {
                let next = if prefix.is_empty() {
                    key.to_string()
                } else {
                    format!("{prefix}.{key}")
                };
                flatten_json(&next, value, out);
            }
        }
        Value::Array(items) => {
            out.insert(
                prefix.to_string(),
                serde_json::to_string(items).unwrap_or_default(),
            );
        }
        Value::String(value) => {
            out.insert(prefix.to_string(), value.clone());
        }
        Value::Bool(value) => {
            out.insert(prefix.to_string(), value.to_string());
        }
        Value::Number(value) => {
            out.insert(prefix.to_string(), value.to_string());
        }
        Value::Null => {
            out.insert(prefix.to_string(), "null".to_string());
        }
    }
}

trait ValueExt {
    fn field_u64(&self, key: &str) -> Result<u64>;
}

impl ValueExt for Value {
    fn field_u64(&self, key: &str) -> Result<u64> {
        self.get(key)
            .and_then(Value::as_u64)
            .ok_or_else(|| format!("missing integer field '{key}'").into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ndjson_stream() {
        let events = parse_stream(
            r#"{"type":"custom","payload":{"message":"one"}}
{"type":"custom","payload":{"message":"two"}}"#,
        )
        .unwrap();
        assert_eq!(events.len(), 2);
        assert_eq!(events[1].payload["message"], "two");
    }

    #[test]
    fn renders_template_from_payload() {
        let event = IncomingEvent::github_issue_opened("repo".into(), 42, "broken".into(), None);
        let rendered = render_template("{repo} #{number}: {title}", &event.template_context());
        assert_eq!(rendered, "repo #42: broken");
    }
}
