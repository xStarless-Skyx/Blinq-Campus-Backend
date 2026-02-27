use revolt_rocket_okapi::revolt_okapi::openapi3::OpenApi;
use rocket::Route;

mod google;

pub fn routes() -> (Vec<Route>, OpenApi) {
    openapi_get_routes_spec![google::google_start, google::google_callback]
}
