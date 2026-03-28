# ALICE-Drone-SaaS

UAV command and control as a Service — autonomous navigation, waypoint management, geofencing, and telemetry via REST API.

## Architecture

```
Client
  |
  v
API Gateway (:8219)
  |
  v
Core Engine (:8119)
  |
  +-- Flight Controller
  +-- Waypoint Planner
  +-- Geofence Manager
  +-- Telemetry Aggregator
```

## Features

- Autonomous point-to-point and multi-waypoint navigation
- Dynamic geofence polygon definition and enforcement
- Real-time telemetry streaming (position, battery, attitude)
- Collision avoidance via SDF-based obstacle representation
- Mission abort and return-to-home failsafe
- Multi-UAV fleet coordination

## API Endpoints

### Core Engine (port 8119)

| Method | Path | Description |
|--------|------|-------------|
| POST | /api/v1/drone/navigate | Command a drone to navigate to a target |
| POST | /api/v1/drone/waypoints | Upload a waypoint mission |
| POST | /api/v1/drone/geofence | Define or update a geofence polygon |
| GET  | /api/v1/drone/telemetry | Retrieve latest telemetry snapshot |
| GET  | /api/v1/drone/stats | Return runtime statistics |
| GET  | /health | Health check |

### Example: Navigate

```bash
curl -X POST http://localhost:8119/api/v1/drone/navigate \
  -H 'Content-Type: application/json' \
  -d '{"drone_id":"d1","target":{"lat":35.6895,"lon":139.6917,"alt_m":50}}'
```

### Example: Upload Waypoints

```bash
curl -X POST http://localhost:8119/api/v1/drone/waypoints \
  -H 'Content-Type: application/json' \
  -d '{"drone_id":"d1","waypoints":[{"lat":35.6895,"lon":139.6917,"alt_m":50},{"lat":35.69,"lon":139.692,"alt_m":60}]}'
```

## License

AGPL-3.0-or-later
