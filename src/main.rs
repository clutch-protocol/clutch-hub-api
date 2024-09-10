use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use serde::{Deserialize, Serialize};

// Define data structures for requests and responses

#[derive(Serialize, Deserialize)]
struct RideRequest {
    pickup_location: String,
    dropoff_location: String,
    user_id: String,
}

#[derive(Serialize, Deserialize)]
struct RideOffer {
    driver_id: String,
    available_seats: u32,
}

// API endpoint to handle ride requests
async fn ride_request(req: web::Json<RideRequest>) -> impl Responder {
    println!("Received a ride request from user: {}", req.user_id);
    HttpResponse::Ok().json(format!(
        "Ride request from {} to {} received!",
        req.pickup_location, req.dropoff_location
    ))
}

// API endpoint to handle ride offers
async fn ride_offer(offer: web::Json<RideOffer>) -> impl Responder {
    println!("Received a ride offer from driver: {}", offer.driver_id);
    HttpResponse::Ok().json(format!(
        "Ride offer with {} available seats received!",
        offer.available_seats
    ))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/api/ride-request", web::post().to(ride_request))
            .route("/api/ride-offer", web::post().to(ride_offer))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
