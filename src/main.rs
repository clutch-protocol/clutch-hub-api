use std::net::Ipv4Addr;

use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};
use utoipa::{ToSchema, OpenApi};
use utoipa::openapi::security::{ApiKey, ApiKeyValue, SecurityScheme};

#[derive(OpenApi)]
#[openapi(
    paths(ride_request, ride_offer),
    components(schemas(RideRequest, RideOffer)),
    tags(
        (name = "ride-sharing", description = "Ride sharing service API")
    )
)]
struct ApiDoc;

#[derive(Serialize, Deserialize, ToSchema)]
struct RideRequest {
    pickup_location: String,
    dropoff_location: String,
    user_id: String,
}

#[derive(Serialize, Deserialize, ToSchema)]
struct RideOffer {
    driver_id: String,
    available_seats: u32,
}

// API endpoint to handle ride requests
#[utoipa::path(
    post,
    path = "/api/ride-request",
    request_body = RideRequest,
    responses(
        (status = 200, description = "Ride request received", body = String)
    )
)]
async fn ride_request(req: web::Json<RideRequest>) -> impl Responder {
    HttpResponse::Ok().json(format!(
        "Ride request from {} to {} received!",
        req.pickup_location, req.dropoff_location
    ))
}

// API endpoint to handle ride offers
#[utoipa::path(
    post,
    path = "/api/ride-offer",
    request_body = RideOffer,
    responses(
        (status = 200, description = "Ride offer received", body = String)
    )
)]
async fn ride_offer(offer: web::Json<RideOffer>) -> impl Responder {
    HttpResponse::Ok().json(format!(
        "Ride offer with {} available seats received!",
        offer.available_seats
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let api_doc = ApiDoc::openapi();

    HttpServer::new(move || {
        App::new()
            .service(actix_web::web::scope("/api")
                .route("/ride-request", web::post().to(ride_request))
                .route("/ride-offer", web::post().to(ride_offer))
            )
            .service(
                utoipa_swagger_ui::SwaggerUi::new("/swagger-ui/{_:.*}")
                    .url("/api-doc/openapi.json", api_doc.clone())
            )
    })
    .bind((Ipv4Addr::UNSPECIFIED, 8080))?
    .run()
    .await
}
