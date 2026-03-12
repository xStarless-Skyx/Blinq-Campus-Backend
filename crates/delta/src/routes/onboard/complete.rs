use authifier::models::Session;
use once_cell::sync::Lazy;
use regex::Regex;
use revolt_database::{Database, User};
use revolt_models::v0;
use revolt_result::{create_error, Result};

use rocket::{serde::json::Json, State};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// Regex for valid usernames
///
/// Block zero width space
/// Block lookalike characters
pub static RE_USERNAME: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^(\p{L}|[\d_.@+-])+$").unwrap());

/// # New User Data
#[derive(Validate, Serialize, Deserialize, JsonSchema)]
pub struct DataOnboard {
    /// New username which will be used to identify the user on the platform
    #[validate(length(min = 2, max = 64), regex = "RE_USERNAME")]
    username: String,
}

/// # Complete Onboarding
///
/// This sets a new username, completes onboarding and allows a user to start using Revolt.
#[openapi(tag = "Onboarding")]
#[post("/complete", data = "<data>")]
pub async fn complete(
    db: &State<Database>,
    session: Session,
    user: Option<User>,
    data: Json<DataOnboard>,
) -> Result<Json<v0::User>> {
    if user.is_some() {
        return Err(create_error!(AlreadyOnboarded));
    }

    let authifier = db.inner().clone().to_authifier().await;
    let account = authifier
        .database
        .find_account(&session.user_id)
        .await
        .map_err(|_| create_error!(InternalError))?;
    let email = account.email;

    let mut data = data.into_inner();
    data.username = email.clone();
    data.validate().map_err(|error| {
        create_error!(FailedValidation {
            error: error.to_string()
        })
    })?;

    let display_name = email
        .split('@')
        .next()
        .map(|local| {
            local
                .split(|c: char| c == '.' || c == '_' || c == '-' || c == '+')
                .filter(|part| !part.is_empty())
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                        None => String::new(),
                    }
                })
                .filter(|part| !part.is_empty())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .filter(|name| !name.is_empty());

    Ok(Json(
        User::create(
            db,
            email.clone(),
            session.user_id,
            Some(revolt_database::PartialUser {
                display_name,
                ..Default::default()
            }),
        )
        .await?
        .into_self(false)
        .await,
    ))
}
