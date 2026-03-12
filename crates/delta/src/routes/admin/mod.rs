use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod alerts;
mod dm_audit;
mod dms;
mod users;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![
        dms::list_dms,
        dms::list_dm_summaries,
        dms::dm_between_users,
        dms::dm_messages,
        alerts::message_alerts,
        dm_audit::list_dm_audit,
        users::list_users,
    ]
}
