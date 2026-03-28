use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use uuid::Uuid;

#[derive(Debug, Default, Serialize)]
struct Stats {
    navigations: u64,
    waypoint_missions: u64,
    geofence_updates: u64,
    telemetry_polls: u64,
    total_requests: u64,
}

type AppState = Arc<Mutex<Stats>>;

// --- request / response types ---

#[derive(Debug, Deserialize, Serialize)]
struct GeoPoint {
    lat: f64,
    lon: f64,
    alt_m: f64,
}

#[derive(Debug, Deserialize)]
struct NavigateRequest {
    drone_id: String,
    target: GeoPoint,
}

#[derive(Debug, Deserialize)]
struct WaypointsRequest {
    drone_id: String,
    waypoints: Vec<GeoPoint>,
}

#[derive(Debug, Deserialize)]
struct GeofenceRequest {
    drone_id: String,
    polygon: Vec<GeoPoint>,
    max_alt_m: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct TelemetryQuery {
    drone_id: Option<String>,
}

#[derive(Debug, Serialize)]
struct ApiResponse<T: Serialize> {
    ok: bool,
    request_id: String,
    data: T,
}

fn ok<T: Serialize>(data: T) -> Json<ApiResponse<T>> {
    Json(ApiResponse {
        ok: true,
        request_id: Uuid::new_v4().to_string(),
        data,
    })
}

// --- handlers ---

async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "alice-drone-saas-core" }))
}

async fn drone_navigate(
    State(state): State<AppState>,
    Json(req): Json<NavigateRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.navigations += 1;
    s.total_requests += 1;
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "drone_id": req.drone_id,
            "target": req.target,
            "mission_id": Uuid::new_v4().to_string(),
            "eta_seconds": 42,
            "status": "navigating",
        })),
    )
}

async fn drone_waypoints(
    State(state): State<AppState>,
    Json(req): Json<WaypointsRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.waypoint_missions += 1;
    s.total_requests += 1;
    let count = req.waypoints.len();
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "drone_id": req.drone_id,
            "mission_id": Uuid::new_v4().to_string(),
            "waypoint_count": count,
            "total_distance_m": count as f64 * 150.0,
            "status": "mission_uploaded",
        })),
    )
}

async fn drone_geofence(
    State(state): State<AppState>,
    Json(req): Json<GeofenceRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.geofence_updates += 1;
    s.total_requests += 1;
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "drone_id": req.drone_id,
            "vertices": req.polygon.len(),
            "max_alt_m": req.max_alt_m.unwrap_or(120.0),
            "fence_id": Uuid::new_v4().to_string(),
            "status": "active",
        })),
    )
}

async fn drone_telemetry(
    State(state): State<AppState>,
    axum::extract::Query(q): axum::extract::Query<TelemetryQuery>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut s = state.lock().unwrap();
    s.telemetry_polls += 1;
    s.total_requests += 1;
    let id = q.drone_id.unwrap_or_else(|| "d1".to_string());
    (
        StatusCode::OK,
        ok(serde_json::json!({
            "drone_id": id,
            "position": { "lat": 35.6895, "lon": 139.6917, "alt_m": 50.0 },
            "velocity_mps": { "vx": 1.2, "vy": 0.3, "vz": 0.0 },
            "battery_pct": 78,
            "heading_deg": 270.0,
            "status": "airborne",
        })),
    )
}

async fn drone_stats(State(state): State<AppState>) -> Json<ApiResponse<serde_json::Value>> {
    let s = state.lock().unwrap();
    ok(serde_json::json!({
        "navigations": s.navigations,
        "waypoint_missions": s.waypoint_missions,
        "geofence_updates": s.geofence_updates,
        "telemetry_polls": s.telemetry_polls,
        "total_requests": s.total_requests,
    }))
}

// --- main ---

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .init();

    let state: AppState = Arc::new(Mutex::new(Stats::default()));

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/drone/navigate", post(drone_navigate))
        .route("/api/v1/drone/waypoints", post(drone_waypoints))
        .route("/api/v1/drone/geofence", post(drone_geofence))
        .route("/api/v1/drone/telemetry", get(drone_telemetry))
        .route("/api/v1/drone/stats", get(drone_stats))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .unwrap_or_else(|_| "8119".to_string())
        .parse()
        .unwrap_or(8119);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("alice-drone-saas-core listening on {addr}");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
