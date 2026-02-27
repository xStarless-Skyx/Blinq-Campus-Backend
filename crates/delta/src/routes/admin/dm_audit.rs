use revolt_database::{Database, User};
use revolt_result::{create_error, Result};
use rocket::{form::FromForm, serde::json::Json};
use serde::{Deserialize, Serialize};

const COLLECTION: &str = "admin_dm_audit";

#[derive(FromForm, JsonSchema)]
pub struct DmAuditQuery {
    limit: Option<i64>,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct DmAuditEntry {
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

/// # List DM Audit Entries
#[openapi(tag = "Admin")]
#[get("/dm-audit?<query..>")]
pub async fn list_dm_audit(
    db: &rocket::State<Database>,
    user: User,
    query: DmAuditQuery,
) -> Result<Json<Vec<DmAuditEntry>>> {
    if !user.privileged {
        return Err(create_error!(NotPrivileged));
    }

    let limit = query.limit.unwrap_or(100).clamp(1, 1000);

    if let Database::MongoDb(mongo) = db.inner() {
        let options = revolt_database::mongodb::options::FindOptions::builder()
            .sort(revolt_database::mongodb::bson::doc! { "_id": -1_i32 })
            .limit(limit)
            .build();

        let entries = mongo
            .find_with_options::<_, DmAuditEntry>(
                COLLECTION,
                revolt_database::mongodb::bson::doc! {},
                options,
            )
            .await
            .map_err(|_| create_error!(InternalError))?;

        return Ok(Json(entries));
    }

    Ok(Json(vec![]))
}
