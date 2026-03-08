use std::sync::Arc;

use crate::Result;
use crate::config::AppConfig;
use crate::discord::DiscordClient;
use crate::events::{IncomingEvent, MessageFormat, render_template};

pub struct Router {
    config: Arc<AppConfig>,
}

impl Router {
    pub fn new(config: Arc<AppConfig>) -> Self {
        Self { config }
    }

    pub async fn dispatch(&self, event: &IncomingEvent, discord: &DiscordClient) -> Result<()> {
        let (channel, _format, content) = self.preview(event)?;
        discord.send_message(&channel, &content).await
    }

    pub fn preview(&self, event: &IncomingEvent) -> Result<(String, MessageFormat, String)> {
        let route = self.config.route(event.canonical_kind());
        let channel = event
            .channel
            .clone()
            .or_else(|| route.and_then(|route| route.channel.clone()))
            .or_else(|| self.config.defaults.channel.clone())
            .ok_or_else(|| format!("no channel configured for event {}", event.canonical_kind()))?;
        let format = event
            .format
            .clone()
            .or_else(|| route.and_then(|route| route.format.clone()))
            .unwrap_or_else(|| self.config.defaults.format.clone());
        let content = if let Some(template) = event
            .template
            .as_deref()
            .or_else(|| route.and_then(|route| route.template.as_deref()))
        {
            render_template(template, &event.template_context())
        } else {
            event.render_default(&format)?
        };
        Ok((channel, format, content))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DefaultsConfig, RouteConfig};

    #[test]
    fn preview_uses_route_overrides() {
        let mut config = AppConfig {
            defaults: DefaultsConfig {
                channel: Some("default".into()),
                format: MessageFormat::Compact,
            },
            ..AppConfig::default()
        };
        config.routes.insert(
            "custom".into(),
            RouteConfig {
                channel: Some("route".into()),
                format: Some(MessageFormat::Alert),
                template: None,
            },
        );
        let router = Router::new(Arc::new(config));
        let event = IncomingEvent::custom(None, "wake up".into());

        let (channel, format, content) = router.preview(&event).unwrap();
        assert_eq!(channel, "route");
        assert_eq!(format, MessageFormat::Alert);
        assert_eq!(content, "🚨 wake up");
    }
}
