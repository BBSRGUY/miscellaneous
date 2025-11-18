"""
SQLite database layer for persistence.

All operations use proper transactions and foreign key constraints.
"""

import json
import sqlite3
from contextlib import contextmanager
from pathlib import Path
from typing import Any, Iterator

from ganita.models import AuditLog, Discovery


class Database:
    """SQLite database manager with WAL mode and foreign keys."""

    def __init__(self, db_path: Path | str):
        self.db_path = Path(db_path)
        self.db_path.parent.mkdir(parents=True, exist_ok=True)
        self._init_db()

    def _init_db(self) -> None:
        """Initialize database with schema."""
        with self.connect() as conn:
            conn.execute("PRAGMA journal_mode=WAL")
            conn.execute("PRAGMA foreign_keys=ON")
            self._create_tables(conn)

    @contextmanager
    def connect(self) -> Iterator[sqlite3.Connection]:
        """Context manager for database connections."""
        conn = sqlite3.connect(self.db_path)
        conn.row_factory = sqlite3.Row
        try:
            yield conn
            conn.commit()
        except Exception:
            conn.rollback()
            raise
        finally:
            conn.close()

    def _create_tables(self, conn: sqlite3.Connection) -> None:
        """Create all tables."""

        # Runs table
        conn.execute("""
            CREATE TABLE IF NOT EXISTS runs (
                run_id INTEGER PRIMARY KEY,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                orchestrator_ver TEXT NOT NULL,
                code_hash TEXT NOT NULL,
                config_json TEXT NOT NULL
            )
        """)

        # Module execution tracking
        conn.execute("""
            CREATE TABLE IF NOT EXISTS run_modules (
                id INTEGER PRIMARY KEY,
                run_id INTEGER NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
                module_name TEXT NOT NULL,
                started_at TEXT NOT NULL,
                finished_at TEXT,
                status TEXT NOT NULL,
                log_path TEXT
            )
        """)

        # Discoveries
        conn.execute("""
            CREATE TABLE IF NOT EXISTS discoveries (
                discovery_id INTEGER PRIMARY KEY,
                run_id INTEGER NOT NULL REFERENCES runs(run_id) ON DELETE CASCADE,
                module_name TEXT NOT NULL,
                stage TEXT NOT NULL,
                value_repr TEXT NOT NULL,
                ulp_bits INTEGER NOT NULL,
                interval_center TEXT NOT NULL,
                interval_radius TEXT NOT NULL,
                accel_used TEXT NOT NULL,
                cert_strategy TEXT NOT NULL,
                params_json TEXT NOT NULL,
                env_json TEXT NOT NULL,
                rank_score REAL NOT NULL DEFAULT 0.0,
                created_at TEXT NOT NULL,
                UNIQUE (run_id, module_name, value_repr)
            )
        """)

        # Discovery traces
        conn.execute("""
            CREATE TABLE IF NOT EXISTS discovery_traces (
                trace_id INTEGER PRIMARY KEY,
                discovery_id INTEGER NOT NULL REFERENCES discoveries(discovery_id) ON DELETE CASCADE,
                trace_path TEXT NOT NULL,
                num_points INTEGER NOT NULL,
                meta_json TEXT NOT NULL
            )
        """)

        # Recognition attempts
        conn.execute("""
            CREATE TABLE IF NOT EXISTS recognition (
                recognition_id INTEGER PRIMARY KEY,
                discovery_id INTEGER NOT NULL REFERENCES discoveries(discovery_id) ON DELETE CASCADE,
                basis_name TEXT NOT NULL,
                target_precision INTEGER NOT NULL,
                success INTEGER NOT NULL,
                relation_tex TEXT,
                relation_json TEXT,
                height INTEGER,
                residual_log10 REAL,
                attempted_at TEXT NOT NULL
            )
        """)

        # Cross-validation
        conn.execute("""
            CREATE TABLE IF NOT EXISTS cross_validation (
                cv_id INTEGER PRIMARY KEY,
                discovery_id INTEGER NOT NULL REFERENCES discoveries(discovery_id) ON DELETE CASCADE,
                other_module TEXT NOT NULL,
                tolerance_str TEXT NOT NULL,
                success INTEGER NOT NULL,
                details_json TEXT NOT NULL,
                checked_at TEXT NOT NULL
            )
        """)

        # Audit logs
        conn.execute("""
            CREATE TABLE IF NOT EXISTS audit (
                audit_id INTEGER PRIMARY KEY,
                ts TEXT NOT NULL,
                actor TEXT NOT NULL,
                action TEXT NOT NULL,
                subject_type TEXT NOT NULL,
                subject_id INTEGER NOT NULL,
                payload_json TEXT NOT NULL
            )
        """)

        # Create indexes
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_disc_stage_score ON discoveries(stage, rank_score DESC)"
        )
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_rec_discovery ON recognition(discovery_id)"
        )
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_cv_discovery ON cross_validation(discovery_id)"
        )

    def create_run(self, orchestrator_ver: str, code_hash: str, config: dict[str, Any]) -> int:
        """Create a new run."""
        from datetime import datetime

        with self.connect() as conn:
            cur = conn.execute(
                """
                INSERT INTO runs (started_at, orchestrator_ver, code_hash, config_json)
                VALUES (?, ?, ?, ?)
            """,
                (datetime.utcnow().isoformat(), orchestrator_ver, code_hash, json.dumps(config)),
            )
            run_id = cur.lastrowid
            assert run_id is not None
            return run_id

    def finish_run(self, run_id: int) -> None:
        """Mark run as finished."""
        from datetime import datetime

        with self.connect() as conn:
            conn.execute(
                "UPDATE runs SET finished_at = ? WHERE run_id = ?",
                (datetime.utcnow().isoformat(), run_id),
            )

    def insert_discovery(self, run_id: int, disc: Discovery) -> int:
        """Insert a discovery and return its ID."""
        with self.connect() as conn:
            # Get final interval
            final = disc.evidence.interval_trace[-1] if disc.evidence.interval_trace else None
            center = final.center if final else disc.value_repr
            radius = final.radius if final else "0"

            cur = conn.execute(
                """
                INSERT INTO discoveries (
                    run_id, module_name, stage, value_repr, ulp_bits,
                    interval_center, interval_radius, accel_used, cert_strategy,
                    params_json, env_json, rank_score, created_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
                (
                    run_id,
                    disc.method,
                    disc.stage.value,
                    disc.value_repr,
                    disc.ulp_bits,
                    center,
                    radius,
                    json.dumps(disc.evidence.accel_used),
                    disc.evidence.cert_strategy,
                    json.dumps(disc.params),
                    disc.env.model_dump_json(),
                    disc.rank_score,
                    disc.created_at,
                ),
            )
            discovery_id = cur.lastrowid
            assert discovery_id is not None
            disc.discovery_id = discovery_id

            # Log audit
            self._log_audit(
                conn,
                action="insert",
                subject_type="discovery",
                subject_id=discovery_id,
                payload={"method": disc.method, "stage": disc.stage.value},
            )

            return discovery_id

    def update_discovery_stage(
        self, discovery_id: int, new_stage: str, actor: str = "system"
    ) -> None:
        """Update discovery stage and log audit."""
        with self.connect() as conn:
            conn.execute(
                "UPDATE discoveries SET stage = ? WHERE discovery_id = ?",
                (new_stage, discovery_id),
            )
            self._log_audit(
                conn,
                action="promote_stage",
                subject_type="discovery",
                subject_id=discovery_id,
                payload={"new_stage": new_stage},
                actor=actor,
            )

    def add_recognition(
        self,
        discovery_id: int,
        basis_name: str,
        precision: int,
        success: bool,
        relation_tex: str | None = None,
        relation_json: dict[str, Any] | None = None,
        height: int | None = None,
        residual_log10: float | None = None,
    ) -> None:
        """Record a recognition attempt."""
        from datetime import datetime

        with self.connect() as conn:
            conn.execute(
                """
                INSERT INTO recognition (
                    discovery_id, basis_name, target_precision, success,
                    relation_tex, relation_json, height, residual_log10, attempted_at
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            """,
                (
                    discovery_id,
                    basis_name,
                    precision,
                    1 if success else 0,
                    relation_tex,
                    json.dumps(relation_json) if relation_json else None,
                    height,
                    residual_log10,
                    datetime.utcnow().isoformat(),
                ),
            )

    def add_cross_validation(
        self,
        discovery_id: int,
        other_module: str,
        tolerance: str,
        success: bool,
        details: dict[str, Any],
    ) -> None:
        """Record cross-validation attempt."""
        from datetime import datetime

        with self.connect() as conn:
            conn.execute(
                """
                INSERT INTO cross_validation (
                    discovery_id, other_module, tolerance_str, success, details_json, checked_at
                )
                VALUES (?, ?, ?, ?, ?, ?)
            """,
                (
                    discovery_id,
                    other_module,
                    tolerance,
                    1 if success else 0,
                    json.dumps(details),
                    datetime.utcnow().isoformat(),
                ),
            )

    def query_top(
        self, stage: str | None = None, limit: int = 20, run_id: int | None = None
    ) -> list[dict[str, Any]]:
        """Query top discoveries by rank."""
        with self.connect() as conn:
            query = "SELECT * FROM discoveries WHERE 1=1"
            params: list[Any] = []

            if stage:
                query += " AND stage = ?"
                params.append(stage)

            if run_id:
                query += " AND run_id = ?"
                params.append(run_id)

            query += " ORDER BY rank_score DESC LIMIT ?"
            params.append(limit)

            cur = conn.execute(query, params)
            return [dict(row) for row in cur.fetchall()]

    def get_discovery(self, discovery_id: int) -> dict[str, Any] | None:
        """Get a single discovery by ID."""
        with self.connect() as conn:
            cur = conn.execute(
                "SELECT * FROM discoveries WHERE discovery_id = ?", (discovery_id,)
            )
            row = cur.fetchone()
            return dict(row) if row else None

    def get_recognition_attempts(self, discovery_id: int) -> list[dict[str, Any]]:
        """Get all recognition attempts for a discovery."""
        with self.connect() as conn:
            cur = conn.execute(
                "SELECT * FROM recognition WHERE discovery_id = ? ORDER BY attempted_at DESC",
                (discovery_id,),
            )
            return [dict(row) for row in cur.fetchall()]

    def _log_audit(
        self,
        conn: sqlite3.Connection,
        action: str,
        subject_type: str,
        subject_id: int,
        payload: dict[str, Any],
        actor: str = "system",
    ) -> None:
        """Log an audit entry."""
        from datetime import datetime

        conn.execute(
            """
            INSERT INTO audit (ts, actor, action, subject_type, subject_id, payload_json)
            VALUES (?, ?, ?, ?, ?, ?)
        """,
            (
                datetime.utcnow().isoformat(),
                actor,
                action,
                subject_type,
                subject_id,
                json.dumps(payload),
            ),
        )

    def get_run_stats(self, run_id: int) -> dict[str, Any]:
        """Get statistics for a run."""
        with self.connect() as conn:
            cur = conn.execute(
                """
                SELECT
                    COUNT(*) as total_discoveries,
                    SUM(CASE WHEN stage = 'certified' THEN 1 ELSE 0 END) as certified,
                    SUM(CASE WHEN stage = 'recognized' THEN 1 ELSE 0 END) as recognized,
                    SUM(CASE WHEN stage = 'cross_validated' THEN 1 ELSE 0 END) as cross_validated,
                    AVG(rank_score) as avg_score,
                    MAX(rank_score) as max_score
                FROM discoveries
                WHERE run_id = ?
            """,
                (run_id,),
            )
            row = cur.fetchone()
            return dict(row) if row else {}
