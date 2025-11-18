"""
FastAPI web service for Ganit-A.

Provides REST API and WebSocket support for real-time updates.
"""

from __future__ import annotations

import asyncio
import json
import logging
from pathlib import Path
from typing import Any

from fastapi import FastAPI, HTTPException, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import HTMLResponse
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

from ganita import __version__
from ganita.models import ModuleConfig, RunConfig
from ganita.orchestrator import Orchestrator
from ganita.storage.db import Database

logger = logging.getLogger(__name__)

# FastAPI app
app = FastAPI(
    title="Ganit-A API",
    description="Mathematical Discovery Engine API",
    version=__version__,
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Global state
db_path = Path("ganita_data/math_discovery.db")
artifacts_path = Path("ganita_data/artifacts")
active_runs: dict[int, Orchestrator] = {}
ws_clients: list[WebSocket] = []


# Request/Response models
class RunRequest(BaseModel):
    target_precision_digits: int = 600
    wallclock_limit_minutes: float = 30.0
    random_seed: int = 42
    max_workers: int = 4
    modules: list[dict[str, Any]] = []


class DiscoveryResponse(BaseModel):
    discovery_id: int
    value_repr: str
    method: str
    stage: str
    rank_score: float
    ulp_bits: int
    params: dict[str, Any]


# API Routes


@app.get("/")
async def root() -> dict[str, str]:
    """API root endpoint."""
    return {
        "name": "Ganit-A API",
        "version": __version__,
        "description": "Mathematical Discovery Engine",
    }


@app.get("/health")
async def health() -> dict[str, str]:
    """Health check endpoint."""
    return {"status": "healthy", "version": __version__}


@app.post("/runs/start")
async def start_run(request: RunRequest) -> dict[str, Any]:
    """Start a new discovery run."""

    # Create configuration
    modules = []
    for mod_dict in request.modules:
        modules.append(ModuleConfig.model_validate(mod_dict))

    if not modules:
        # Default modules
        modules = [
            ModuleConfig(name="series_products", enabled=True, budget_minutes=10.0),
            ModuleConfig(name="continued_fractions", enabled=True, budget_minutes=10.0),
            ModuleConfig(name="fixed_points", enabled=True, budget_minutes=10.0),
        ]

    config = RunConfig(
        target_precision_digits=request.target_precision_digits,
        wallclock_limit_minutes=request.wallclock_limit_minutes,
        random_seed=request.random_seed,
        max_workers=request.max_workers,
        modules=modules,
    )

    # Create orchestrator
    orchestrator = Orchestrator(config, db_path, artifacts_path)

    # Run in background
    asyncio.create_task(_run_exploration_background(orchestrator))

    return {
        "status": "started",
        "message": "Discovery run started in background",
    }


@app.get("/runs")
async def list_runs() -> list[dict[str, Any]]:
    """List all runs."""
    if not db_path.exists():
        return []

    db = Database(db_path)
    with db.connect() as conn:
        cur = conn.execute(
            """
            SELECT run_id, started_at, finished_at, orchestrator_ver
            FROM runs
            ORDER BY run_id DESC
            LIMIT 50
        """
        )
        rows = cur.fetchall()
        return [dict(row) for row in rows]


@app.get("/runs/{run_id}/stats")
async def get_run_stats(run_id: int) -> dict[str, Any]:
    """Get statistics for a specific run."""
    if not db_path.exists():
        raise HTTPException(status_code=404, detail="Database not found")

    db = Database(db_path)
    stats = db.get_run_stats(run_id)

    if not stats:
        raise HTTPException(status_code=404, detail=f"Run {run_id} not found")

    return stats


@app.get("/discoveries")
async def list_discoveries(
    run_id: int | None = None,
    stage: str | None = None,
    limit: int = 20,
    offset: int = 0,
) -> list[dict[str, Any]]:
    """List discoveries with filtering."""
    if not db_path.exists():
        return []

    db = Database(db_path)

    query = "SELECT * FROM discoveries WHERE 1=1"
    params: list[Any] = []

    if run_id:
        query += " AND run_id = ?"
        params.append(run_id)

    if stage:
        query += " AND stage = ?"
        params.append(stage)

    query += " ORDER BY rank_score DESC LIMIT ? OFFSET ?"
    params.extend([limit, offset])

    with db.connect() as conn:
        cur = conn.execute(query, params)
        rows = cur.fetchall()
        return [dict(row) for row in rows]


@app.get("/discoveries/{discovery_id}")
async def get_discovery(discovery_id: int) -> dict[str, Any]:
    """Get detailed information about a discovery."""
    if not db_path.exists():
        raise HTTPException(status_code=404, detail="Database not found")

    db = Database(db_path)
    disc = db.get_discovery(discovery_id)

    if not disc:
        raise HTTPException(status_code=404, detail=f"Discovery {discovery_id} not found")

    # Add recognition attempts
    recognition = db.get_recognition_attempts(discovery_id)
    disc["recognition_attempts"] = recognition

    return disc


@app.get("/discoveries/{discovery_id}/trace")
async def get_discovery_trace(discovery_id: int) -> list[dict[str, Any]]:
    """Get interval trace for a discovery."""
    if not db_path.exists():
        raise HTTPException(status_code=404, detail="Database not found")

    # Load trace from artifacts
    from ganita.storage.artifacts import ArtifactStore

    artifacts = ArtifactStore(artifacts_path)

    try:
        # Find trace path
        db = Database(db_path)
        with db.connect() as conn:
            cur = conn.execute(
                "SELECT trace_path FROM discovery_traces WHERE discovery_id = ?",
                (discovery_id,),
            )
            row = cur.fetchone()

            if not row:
                raise HTTPException(status_code=404, detail="Trace not found")

            trace_path = row["trace_path"]

        # Load trace
        samples = artifacts.read_trace(trace_path)
        return [s.model_dump() for s in samples]

    except Exception as e:
        raise HTTPException(status_code=500, detail=str(e))


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket) -> None:
    """WebSocket endpoint for real-time updates."""
    await websocket.accept()
    ws_clients.append(websocket)

    try:
        while True:
            # Keep connection alive
            await websocket.receive_text()

    except WebSocketDisconnect:
        ws_clients.remove(websocket)


async def _run_exploration_background(orchestrator: Orchestrator) -> None:
    """Run exploration in background and send updates via WebSocket."""
    try:
        # Notify clients
        await _broadcast_ws({"type": "run_started", "message": "Discovery run started"})

        # Run exploration
        run_id = orchestrator.run()

        # Notify completion
        stats = orchestrator.db.get_run_stats(run_id)
        await _broadcast_ws(
            {
                "type": "run_completed",
                "run_id": run_id,
                "stats": stats,
            }
        )

    except Exception as e:
        logger.error(f"Background run failed: {e}", exc_info=True)
        await _broadcast_ws({"type": "run_error", "error": str(e)})


async def _broadcast_ws(message: dict[str, Any]) -> None:
    """Broadcast message to all WebSocket clients."""
    if not ws_clients:
        return

    message_str = json.dumps(message)
    disconnected = []

    for client in ws_clients:
        try:
            await client.send_text(message_str)
        except Exception:
            disconnected.append(client)

    # Remove disconnected clients
    for client in disconnected:
        if client in ws_clients:
            ws_clients.remove(client)


# Serve static files for web UI
# Mount this after API routes to avoid conflicts
try:
    ui_path = Path(__file__).parent / "ui" / "dist"
    if ui_path.exists():
        app.mount("/ui", StaticFiles(directory=ui_path, html=True), name="ui")
except Exception as e:
    logger.warning(f"Could not mount UI static files: {e}")


# Main entry point
if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8000, log_level="info")
