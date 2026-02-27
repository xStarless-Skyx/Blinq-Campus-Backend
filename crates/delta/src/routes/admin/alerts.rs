use revolt_database::{Channel, Database, MessageFilter, MessageQuery, MessageTimePeriod, User};
use revolt_models::v0::MessageSort;
use revolt_result::{create_error, Result};
use rocket::{form::FromForm, serde::json::Json, State};
use serde::Serialize;

const DEFAULT_ALERT_KEYWORDS: &[&str] = &[
    "scam",
    "spam",
    "fraud",
    "hate",
    "threat",
    "nude",
    "doxx",
];

#[derive(FromForm, JsonSchema)]
pub struct AlertsQuery {
    limit: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
pub struct MessageAlert {
    pub channel_id: String,
    pub message_id: String,
    pub author_id: String,
    pub content: String,
    pub matched_keywords: Vec<String>,
}

fn alert_keywords() -> Vec<String> {
    let configured = std::env::var("ADMIN_ALERT_KEYWORDS").unwrap_or_default();
    let mut keywords = configured
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_lowercase())
        .collect::<Vec<String>>();

    if keywords.is_empty() {
        keywords = DEFAULT_ALERT_KEYWORDS
            .iter()
            .map(|value| value.to_string())
            .collect();
    }

    keywords
}

/// # Fetch Message Alerts
///
/// Scans recent DM/group messages for configured keyword matches.
#[openapi(tag = "Admin")]
#[get("/alerts/messages?<query..>")]
pub async fn message_alerts(
    db: &State<Database>,
    user: User,
    query: AlertsQuery,
) -> Result<Json<Vec<MessageAlert>>> {
    if !user.privileged {
        return Err(create_error!(NotPrivileged));
    }

    let max_results = query.limit.unwrap_or(100).min(500);
    let keywords = alert_keywords();
    let mut alerts = Vec::new();

    for channel in db.find_all_direct_messages().await? {
        let channel_id = match channel {
            Channel::DirectMessage { id, .. } | Channel::Group { id, .. } => id,
            _ => continue,
        };

        let recent_messages = db
            .fetch_messages(MessageQuery {
                limit: Some(50),
                filter: MessageFilter {
                    channel: Some(channel_id.clone()),
                    ..Default::default()
                },
                time_period: MessageTimePeriod::Absolute {
                    before: None,
                    after: None,
                    sort: Some(MessageSort::Latest),
                },
            })
            .await?;

        for message in recent_messages {
            let content = match message.content {
                Some(content) if !content.is_empty() => content,
                _ => continue,
            };

            let content_lower = content.to_lowercase();
            let matched_keywords = keywords
                .iter()
                .filter(|keyword| content_lower.contains(keyword.as_str()))
                .cloned()
                .collect::<Vec<String>>();

            if matched_keywords.is_empty() {
                continue;
            }

            alerts.push(MessageAlert {
                channel_id: channel_id.clone(),
                message_id: message.id,
                author_id: message.author,
                content,
                matched_keywords,
            });
        }
    }

    alerts.sort_by(|a, b| b.message_id.cmp(&a.message_id));
    alerts.truncate(max_results);

    Ok(Json(alerts))
}
