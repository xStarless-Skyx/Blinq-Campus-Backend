use iso8601_timestamp::Timestamp;
use revolt_database::{Channel, Database, Message};
use revolt_result::{create_error, Result};
use serde::Serialize;
use ulid::Ulid;

const COLLECTION: &str = "admin_dm_audit";

#[derive(Serialize)]
struct DmAuditRecord {
    #[serde(rename = "_id")]
    id: String,
    action: String,
    occurred_at: String,
    actor_id: String,
    channel_id: String,
    message_id: String,
    recipients: Vec<String>,
    author_id: String,
    old_content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    new_content: Option<String>,
}

fn dm_recipients(channel: &Channel) -> Option<Vec<String>> {
    match channel {
        Channel::DirectMessage { recipients, .. } | Channel::Group { recipients, .. } => {
            Some(recipients.clone())
        }
        _ => None,
    }
}

pub async fn log_edit(
    db: &Database,
    actor_id: &str,
    channel: &Channel,
    message: &Message,
    new_content: Option<&str>,
) -> Result<()> {
    let recipients = match dm_recipients(channel) {
        Some(recipients) => recipients,
        None => return Ok(()),
    };

    if let Database::MongoDb(mongo) = db {
        let record = DmAuditRecord {
            id: Ulid::new().to_string(),
            action: "edit".to_string(),
            occurred_at: Timestamp::now_utc().to_string(),
            actor_id: actor_id.to_string(),
            channel_id: message.channel.clone(),
            message_id: message.id.clone(),
            recipients,
            author_id: message.author.clone(),
            old_content: message.content.clone(),
            new_content: new_content.map(str::to_string),
        };

        mongo
            .col::<DmAuditRecord>(COLLECTION)
            .insert_one(record)
            .await
            .map_err(|_| create_error!(InternalError))?;
    }

    Ok(())
}

pub async fn log_delete(
    db: &Database,
    actor_id: &str,
    channel: &Channel,
    message: &Message,
) -> Result<()> {
    let recipients = match dm_recipients(channel) {
        Some(recipients) => recipients,
        None => return Ok(()),
    };

    if let Database::MongoDb(mongo) = db {
        let record = DmAuditRecord {
            id: Ulid::new().to_string(),
            action: "delete".to_string(),
            occurred_at: Timestamp::now_utc().to_string(),
            actor_id: actor_id.to_string(),
            channel_id: message.channel.clone(),
            message_id: message.id.clone(),
            recipients,
            author_id: message.author.clone(),
            old_content: message.content.clone(),
            new_content: None,
        };

        mongo
            .col::<DmAuditRecord>(COLLECTION)
            .insert_one(record)
            .await
            .map_err(|_| create_error!(InternalError))?;
    }

    Ok(())
}
