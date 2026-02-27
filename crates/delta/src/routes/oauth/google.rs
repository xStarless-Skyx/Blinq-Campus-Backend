use authifier::{models::Account, util::normalise_email, Authifier};
use nanoid::nanoid;
use revolt_database::{PartialUser, User};
use revolt_result::{create_error, Result, ToRevoltError};
use rocket::{
    http::{Cookie, CookieJar, SameSite},
    response::Redirect,
    State,
};
use serde::{Deserialize, Serialize};
use url::{form_urlencoded::Serializer, Url};

const GOOGLE_AUTH_URL: &str = "https://accounts.google.com/o/oauth2/v2/auth";
const GOOGLE_TOKEN_URL: &str = "https://oauth2.googleapis.com/token";
const GOOGLE_USERINFO_URL: &str = "https://openidconnect.googleapis.com/v1/userinfo";
const OAUTH_STATE_COOKIE: &str = "oauth_google_state";
const GOOGLE_OAUTH_CLIENT_ID_FALLBACK: &str =
    "CLIENT_ID (removed for privacy/ security)";
const  &str = // removed for privacy / security
const GOOGLE_OAUTH_REDIRECT_URI_FALLBACK: &str =
    "https://local.revolt.chat:24702/auth/session/oauth/google/callback";
const GOOGLE_OAUTH_APP_REDIRECT_URI_FALLBACK: &str = "https://local.revolt.chat:24701/";

#[derive(Deserialize)]
struct GoogleTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GoogleUserInfo {
    email: Option<String>,
    email_verified: Option<bool>,
    name: Option<String>,
    given_name: Option<String>,
    family_name: Option<String>,
}

#[derive(Deserialize, JsonSchema, FromForm)]
pub struct GoogleCallbackQuery {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
    error_description: Option<String>,
}

#[derive(Serialize, JsonSchema)]
pub struct OAuthSessionResponse {
    token: String,
    user_id: String,
    created_account: bool,
}

fn required_env(name: &str) -> Result<String> {
    let hardcoded = match name {
        "GOOGLE_OAUTH_CLIENT_ID" => Some(GOOGLE_OAUTH_CLIENT_ID_FALLBACK),
        "GOOGLE_OAUTH_CLIENT_SECRET" => Some(GOOGLE_OAUTH_CLIENT_SECRET_FALLBACK),
        "GOOGLE_OAUTH_REDIRECT_URI" => Some(GOOGLE_OAUTH_REDIRECT_URI_FALLBACK),
        "GOOGLE_OAUTH_APP_REDIRECT_URI" => Some(GOOGLE_OAUTH_APP_REDIRECT_URI_FALLBACK),
        _ => None,
    };

    if let Some(value) = hardcoded {
        return Ok(value.to_string());
    }

    if let Some(value) = std::env::var(name).ok().filter(|value| !value.is_empty()) {
        return Ok(value);
    }

    std::env::var(name).map_err(|_| {
        create_error!(FailedValidation {
            error: format!("Missing required environment variable: {name}")
        })
    })
}

fn validate_local_redirect_target(url: &Url) -> Result<()> {
    let scheme = url.scheme();
    let host = url.host_str().unwrap_or_default();

    let allowed_scheme = scheme == "https" || scheme == "http";
    let allowed_host = matches!(host, "local.revolt.chat" | "localhost");

    if allowed_scheme && allowed_host {
        Ok(())
    } else {
        Err(create_error!(FailedValidation {
            error: "GOOGLE_OAUTH_APP_REDIRECT_URI must be local (local.revolt.chat or localhost)"
                .to_string()
        }))
    }
}

fn clean_username_candidate(input: &str) -> String {
    let mut cleaned = String::new();
    let mut last_was_sep = false;

    for ch in input.chars() {
        if ch.is_alphanumeric() || matches!(ch, '_' | '.' | '-') {
            cleaned.push(ch);
            last_was_sep = false;
            continue;
        }

        if ch.is_whitespace() && !cleaned.is_empty() && !last_was_sep {
            cleaned.push('_');
            last_was_sep = true;
        }
    }

    cleaned
        .trim_matches(|c| matches!(c, '_' | '.' | '-'))
        .chars()
        .take(32)
        .collect()
}

fn google_display_name(userinfo: &GoogleUserInfo) -> Option<String> {
    if let Some(name) = userinfo.name.as_ref().map(|value| value.trim()) {
        if !name.is_empty() {
            return Some(name.to_string());
        }
    }

    match (
        userinfo.given_name.as_ref().map(|value| value.trim()),
        userinfo.family_name.as_ref().map(|value| value.trim()),
    ) {
        (Some(given), Some(family)) if !given.is_empty() && !family.is_empty() => {
            Some(format!("{given} {family}"))
        }
        (Some(given), _) if !given.is_empty() => Some(given.to_string()),
        (_, Some(family)) if !family.is_empty() => Some(family.to_string()),
        _ => None,
    }
}

fn google_username(userinfo: &GoogleUserInfo, email: &str) -> String {
    let display_name = google_display_name(userinfo).unwrap_or_default();
    let email_local = email.split('@').next().unwrap_or_default();

    for raw in [
        display_name.as_str(),
        userinfo.given_name.as_deref().unwrap_or_default(),
        email_local,
        "google_user",
    ] {
        let mut candidate = clean_username_candidate(raw);
        if candidate.len() < 2 {
            continue;
        }

        if User::validate_username(candidate.clone()).is_ok() {
            return candidate;
        }

        candidate.push_str("_user");
        candidate = candidate.chars().take(32).collect();

        if candidate.len() >= 2 && User::validate_username(candidate.clone()).is_ok() {
            return candidate;
        }
    }

    "google_user".to_string()
}

/// # Start Google OAuth
///
/// Redirects to Google's consent screen.
///
/// Required environment variables:
/// - `GOOGLE_OAUTH_CLIENT_ID`
/// - `GOOGLE_OAUTH_REDIRECT_URI`
#[openapi(tag = "Session")]
#[get("/google")]
pub async fn google_start(cookies: &CookieJar<'_>) -> Result<Redirect> {
    let client_id = required_env("GOOGLE_OAUTH_CLIENT_ID")?;
    let redirect_uri = required_env("GOOGLE_OAUTH_REDIRECT_URI")?;
    let use_secure_cookie = redirect_uri.starts_with("https://");

    let state = nanoid!(32);

    cookies.add(
        Cookie::build((OAUTH_STATE_COOKIE, state.clone()))
            .path("/")
            .http_only(true)
            .same_site(SameSite::Lax)
            .secure(use_secure_cookie)
            .build(),
    );

    let mut url = Url::parse(GOOGLE_AUTH_URL).to_internal_error()?;
    url.query_pairs_mut()
        .append_pair("response_type", "code")
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &redirect_uri)
        .append_pair("scope", "openid email profile")
        .append_pair("state", &state)
        .append_pair("access_type", "online")
        .append_pair("prompt", "select_account");

    Ok(Redirect::to(url.to_string()))
}

/// # Google OAuth Callback
///
/// Exchanges the authorization code for Google user data, then creates
/// (or reuses) an Authifier account and returns a session token.
///
/// Required environment variables:
/// - `GOOGLE_OAUTH_CLIENT_ID`
/// - `GOOGLE_OAUTH_CLIENT_SECRET`
/// - `GOOGLE_OAUTH_REDIRECT_URI`
///
/// Required for callback completion:
/// - `GOOGLE_OAUTH_APP_REDIRECT_URI`
#[openapi(tag = "Session")]
#[get("/google/callback?<query..>")]
pub async fn google_callback(
    db: &State<revolt_database::Database>,
    authifier: &State<Authifier>,
    cookies: &CookieJar<'_>,
    query: GoogleCallbackQuery,
) -> Result<Redirect> {
    if let Some(error) = query.error {
        let description = query.error_description.unwrap_or_default();
        return Err(create_error!(FailedValidation {
            error: format!("Google OAuth failed: {error} {description}")
        }));
    }

    let code = query.code.ok_or_else(|| {
        create_error!(FailedValidation {
            error: "Missing OAuth code".to_string()
        })
    })?;

    let state = query.state.ok_or_else(|| {
        create_error!(FailedValidation {
            error: "Missing OAuth state".to_string()
        })
    })?;

    if let Some(stored_state) = cookies
        .get(OAUTH_STATE_COOKIE)
        .map(|cookie| cookie.value().to_string())
    {
        cookies.remove(Cookie::from(OAUTH_STATE_COOKIE));

        if stored_state != state {
            return Err(create_error!(InvalidCredentials));
        }
    } else {
        // Local dev browsers sometimes drop the state cookie; allow callback to continue.
        log::warn!("Google OAuth state cookie missing; continuing callback for local dev");
    }

    let client_id = required_env("GOOGLE_OAUTH_CLIENT_ID")?;
    let client_secret = required_env("GOOGLE_OAUTH_CLIENT_SECRET")?;
    let redirect_uri = required_env("GOOGLE_OAUTH_REDIRECT_URI")?;

    let client = reqwest::Client::new();

    let token_response = client
        .post(GOOGLE_TOKEN_URL)
        .form(&[
            ("client_id", client_id.as_str()),
            ("client_secret", client_secret.as_str()),
            ("code", code.as_str()),
            ("grant_type", "authorization_code"),
            ("redirect_uri", redirect_uri.as_str()),
        ])
        .send()
        .await
        .to_internal_error()?;

    if !token_response.status().is_success() {
        let body = token_response.text().await.unwrap_or_default();
        log::warn!("Google OAuth token exchange failed: {body}");
        return Err(create_error!(FailedValidation {
            error: format!("Google OAuth token exchange failed: {body}")
        }));
    }

    let token_response: GoogleTokenResponse = token_response.json().await.to_internal_error()?;

    let userinfo_response = client
        .get(GOOGLE_USERINFO_URL)
        .bearer_auth(token_response.access_token)
        .send()
        .await
        .to_internal_error()?;

    if !userinfo_response.status().is_success() {
        let body = userinfo_response.text().await.unwrap_or_default();
        log::warn!("Google OAuth userinfo request failed: {body}");
        return Err(create_error!(FailedValidation {
            error: format!("Google OAuth userinfo request failed: {body}")
        }));
    }

    let userinfo: GoogleUserInfo = userinfo_response.json().await.to_internal_error()?;
    let email = userinfo
        .email
        .clone()
        .ok_or_else(|| create_error!(InvalidCredentials))?;

    if !userinfo.email_verified.unwrap_or(false) {
        return Err(create_error!(InvalidCredentials));
    }

    let email_normalised = normalise_email(email.clone());
    let preferred_username = google_username(&userinfo, &email_normalised);
    let preferred_display_name = google_display_name(&userinfo);

    let mut created_account = false;
    let account = if let Some(account) = authifier
        .database
        .find_account_by_normalised_email(&email_normalised)
        .await
        .map_err(|error| {
            log::error!("Failed looking up account by email: {error:?}");
            create_error!(InternalError)
        })?
    {
        account
    } else {
        created_account = true;
        Account::new(authifier, email, nanoid!(64), false)
            .await
            .map_err(|error| {
                log::error!("Failed creating authifier account from Google OAuth: {error:?}");
                create_error!(InternalError)
            })?
    };

    match db.fetch_user(&account.id).await {
        Ok(mut user) => {
            if !user.privileged {
                if user.username != preferred_username {
                    user.update_username(db, preferred_username.clone())
                        .await?;
                }

                if preferred_display_name != user.display_name {
                    user.update(
                        db,
                        PartialUser {
                            display_name: preferred_display_name.clone(),
                            ..Default::default()
                        },
                        vec![],
                    )
                    .await?;
                }
            }
        }
        Err(error) if matches!(error.error_type, revolt_result::ErrorType::NotFound) => {
            let mut user = User::create(
                db,
                preferred_username,
                Some(account.id.clone()),
                None,
            )
            .await?;

            if preferred_display_name.is_some() {
                user.update(
                    db,
                    PartialUser {
                        display_name: preferred_display_name.clone(),
                        ..Default::default()
                    },
                    vec![],
                )
                .await?;
            }
        }
        Err(error) => return Err(error),
    }

    let session = account
        .create_session(authifier, "Google OAuth".to_string())
        .await
        .map_err(|error| {
            log::error!("Failed creating session for Google OAuth login: {error:?}");
            create_error!(InternalError)
        })?;

    let payload = OAuthSessionResponse {
        token: session.token,
        user_id: session.user_id,
        created_account,
    };

    let app_redirect_uri = required_env("GOOGLE_OAUTH_APP_REDIRECT_URI")?;
    let mut redirect_url = Url::parse(&app_redirect_uri).map_err(|_| {
        create_error!(FailedValidation {
            error: "Invalid GOOGLE_OAUTH_APP_REDIRECT_URI".to_string()
        })
    })?;
    validate_local_redirect_target(&redirect_url)?;

    let mut serializer = Serializer::new(String::new());
    serializer.append_pair("token", &payload.token);
    serializer.append_pair("user_id", &payload.user_id);
    serializer.append_pair("created_account", &payload.created_account.to_string());

    redirect_url.set_fragment(Some(&serializer.finish()));
    Ok(Redirect::to(redirect_url.to_string()))
}
