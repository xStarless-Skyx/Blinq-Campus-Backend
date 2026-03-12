use revolt_database::{
    Channel, Database, Message, MessageFilter, MessageQuery, MessageTimePeriod, User,
};
use revolt_models::v0::{self, MessageSort};
use revolt_result::{create_error, Result};
use rocket::{form::FromForm, serde::json::Json, State};
use serde::Serialize;
use validator::Validate;

#[derive(FromForm, JsonSchema)]
pub struct DmSummaryQuery {
    q: Option<String>,
    limit: Option<usize>,
}

#[derive(Serialize, JsonSchema)]
pub struct DmSummary {
    channel_id: String,
    user_a_id: String,
    user_a_name: String,
    user_a_avatar: Option<v0::File>,
    user_b_id: String,
    user_b_name: String,
    user_b_avatar: Option<v0::File>,
    last_message_id: Option<String>,
    last_message_preview: Option<String>,
}

fn parse_search(input: &str) -> (Vec<String>, Option<String>) {
    let mut text_query = None;
    let mut remaining = input.to_string();

    if let Some(start) = input.find("%(") {
        if let Some(end_rel) = input[start + 2..].find(')') {
            let end = start + 2 + end_rel;
            let raw = input[start + 2..end].trim();
            if !raw.is_empty() {
                text_query = Some(raw.to_lowercase());
            }
            remaining.replace_range(start..=end, "");
        }
    }

    let terms = remaining
        .split_whitespace()
        .map(|term| term.trim().to_lowercase())
        .filter(|term| !term.is_empty())
        .collect::<Vec<String>>();

    (terms, text_query)
}

/// # List All DM Channels
#[openapi(tag = "Admin")]
#[get("/dms")]
pub async fn list_dms(db: &State<Database>, user: User) -> Result<Json<Vec<v0::Channel>>> {
    if !user.privileged {
        return Err(create_error!(NotPrivileged));
    }

    db.find_all_direct_messages()
        .await
        .map(|channels| channels.into_iter().map(Into::into).collect())
        .map(Json)
}

/// # Fetch DM Between Two Users
#[openapi(tag = "Admin")]
#[get("/dms/<user_a>/<user_b>")]
pub async fn dm_between_users(
    db: &State<Database>,
    user: User,
    user_a: String,
    user_b: String,
) -> Result<Json<v0::Channel>> {
    if !user.privileged {
        return Err(create_error!(NotPrivileged));
    }

    db.find_direct_message_channel(&user_a, &user_b)
        .await
        .map(Into::into)
        .map(Json)
}

/// # List DM Summaries
///
/// Search syntax:
/// - `alice bob` => match usernames in the pair.
/// - `%(keyword)` => search message text in DMs.
#[openapi(tag = "Admin")]
#[get("/dms/summary?<query..>")]
pub async fn list_dm_summaries(
    db: &State<Database>,
    user: User,
    query: DmSummaryQuery,
) -> Result<Json<Vec<DmSummary>>> {
    if !user.privileged {
        return Err(create_error!(NotPrivileged));
    }

    let limit = query.limit.unwrap_or(250).min(1000);
    let (terms, text_query) = parse_search(&query.q.unwrap_or_default());

    let mut summaries = Vec::new();
    for channel in db.find_all_direct_messages().await? {
        let (channel_id, recipients) = match channel {
            Channel::DirectMessage { id, recipients, .. } => (id, recipients),
            _ => continue,
        };

        if recipients.len() != 2 {
            continue;
        }

        let users = db.fetch_users(&recipients).await?;
        if users.len() != 2 {
            continue;
        }

        let user_a = &users[0];
        let user_b = &users[1];
        let pair_text = format!(
            "{} {} {} {}",
            user_a.username,
            user_a.display_name.clone().unwrap_or_default(),
            user_b.username,
            user_b.display_name.clone().unwrap_or_default()
        )
        .to_lowercase();

        if !terms.iter().all(|term| pair_text.contains(term)) {
            continue;
        }

        let mut message_filter = MessageFilter {
            channel: Some(channel_id.clone()),
            ..Default::default()
        };
        if let Some(text) = text_query.clone() {
            message_filter.query = Some(text);
        }

        let latest = db
            .fetch_messages(MessageQuery {
                limit: Some(1),
                filter: message_filter,
                time_period: MessageTimePeriod::Absolute {
                    before: None,
                    after: None,
                    sort: Some(MessageSort::Latest),
                },
            })
            .await?;

        if text_query.is_some() && latest.is_empty() {
            continue;
        }

        let latest_message = latest.into_iter().next();

        summaries.push(DmSummary {
            channel_id,
            user_a_id: user_a.id.clone(),
            user_a_name: user_a.display_name.clone().unwrap_or(user_a.username.clone()),
            user_a_avatar: user_a.avatar.clone().map(Into::into),
            user_b_id: user_b.id.clone(),
            user_b_name: user_b.display_name.clone().unwrap_or(user_b.username.clone()),
            user_b_avatar: user_b.avatar.clone().map(Into::into),
            last_message_id: latest_message.as_ref().map(|m| m.id.clone()),
            last_message_preview: latest_message
                .and_then(|m| m.content)
                .map(|content| content.chars().take(180).collect()),
        });
    }

    summaries.sort_by(|a, b| b.last_message_id.cmp(&a.last_message_id));
    summaries.truncate(limit);

    Ok(Json(summaries))
}

/// # Fetch DM Messages (Admin)
///
/// Fetch messages from a DM channel as an admin.
#[openapi(tag = "Admin")]
#[get("/dms/<channel_id>/messages?<options..>")]
pub async fn dm_messages(
    db: &State<Database>,
    user: User,
    channel_id: String,
    options: v0::OptionsQueryMessages,
) -> Result<Json<v0::BulkMessageResponse>> {
    if !user.privileged {
        return Err(create_error!(NotPrivileged));
    }

    options.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    if let Some(MessageSort::Relevance) = options.sort {
        return Err(create_error!(InvalidOperation));
    }

    let channel = db.fetch_channel(&channel_id).await?;
    match channel {
        Channel::DirectMessage { .. } => {}
        _ => return Err(create_error!(InvalidOperation)),
    }

    let v0::OptionsQueryMessages {
        limit,
        before,
        after,
        sort,
        nearby,
        include_users,
    } = options;

    Message::fetch_with_users(
        db,
        MessageQuery {
            filter: MessageFilter {
                channel: Some(channel_id),
                ..Default::default()
            },
            time_period: if let Some(nearby) = nearby {
                MessageTimePeriod::Relative { nearby }
            } else {
                MessageTimePeriod::Absolute {
                    before,
                    after,
                    sort,
                }
            },
            limit,
        },
        &user,
        include_users.or(Some(true)),
        None,
    )
    .await
    .map(Json)
}
