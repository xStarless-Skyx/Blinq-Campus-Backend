use std::collections::HashMap;

use authifier::models::Account;
use revolt_database::{Database, User};
use revolt_database::mongodb::{bson::doc, options::FindOptions};
use revolt_result::{create_error, Result};
use rocket::{form::FromForm, serde::json::Json, State};
use schemars::JsonSchema;
use serde::Serialize;

#[derive(FromForm, JsonSchema)]
pub struct UsersQuery {
    limit: Option<i64>,
}

#[derive(Serialize, JsonSchema)]
pub struct AdminUserEntry {
    #[serde(rename = "_id")]
    pub id: String,
    pub username: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
}

/// # List Users (Admin)
///
/// Returns users with email addresses for admin-only tooling.
#[openapi(tag = "Admin")]
#[get("/users?<query..>")]
pub async fn list_users(
    db: &State<Database>,
    user: User,
    query: UsersQuery,
) -> Result<Json<Vec<AdminUserEntry>>> {
    if !user.privileged {
        return Err(create_error!(NotPrivileged));
    }

    let limit = query.limit.unwrap_or(5000).clamp(1, 20_000);

    let entries = match db.inner() {
        Database::MongoDb(mongo) => {
            let options = FindOptions::builder().limit(Some(limit)).build();
            let users: Vec<revolt_database::User> = mongo
                .find_with_options("users", doc! {}, options)
                .await
                .map_err(|_| create_error!(InternalError))?;

            let accounts: Vec<Account> = mongo
                .find("accounts", doc! {})
                .await
                .map_err(|_| create_error!(InternalError))?;

            let mut email_by_id: HashMap<String, String> = HashMap::new();
            for account in accounts {
                email_by_id.insert(account.id, account.email);
            }

            let mut entries = users
                .into_iter()
                .map(|u| {
                    let email = email_by_id.get(&u.id).cloned();
                    AdminUserEntry {
                        id: u.id,
                        username: u.username,
                        display_name: u.display_name,
                        email,
                    }
                })
                .collect::<Vec<_>>();

            entries.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));
            entries
        }
        Database::Reference(reference) => {
            let users = reference.users.lock().await;
            let mut entries = users
                .values()
                .map(|u| AdminUserEntry {
                    id: u.id.clone(),
                    username: u.username.clone(),
                    display_name: u.display_name.clone(),
                    email: None,
                })
                .collect::<Vec<_>>();

            entries.sort_by(|a, b| a.username.to_lowercase().cmp(&b.username.to_lowercase()));
            entries
        }
    };

    Ok(Json(entries))
}
