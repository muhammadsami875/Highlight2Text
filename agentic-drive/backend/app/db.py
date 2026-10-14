"""Storage layer with two backends, chosen automatically:

- Postgres, when DATABASE_URL (or Vercel Postgres's POSTGRES_URL) is set — the right
  choice in production, since a Vercel serverless function's filesystem is read-only
  and nothing written to /tmp survives past the current invocation.
- SQLite, otherwise — zero-setup for local dev (./run.sh) and good enough for a quick
  demo deploy, with the caveat that data resets on every cold start when actually run
  on Vercel without a database configured (the app still works, it just forgets).

Both backends share the exact same schema and call surface; every other module in the
app only calls the functions below and never touches SQL directly.
"""
import json
import sqlite3
import time
import uuid
from contextlib import contextmanager

from . import config

SQLITE_SCHEMA = """
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY,
    filename TEXT NOT NULL,
    text TEXT NOT NULL,
    summary TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',
    cluster TEXT NOT NULL DEFAULT 'Unsorted',
    cluster_reason TEXT NOT NULL DEFAULT '',
    doc_type TEXT NOT NULL DEFAULT 'document',
    source_file_ids TEXT NOT NULL DEFAULT '[]',
    created_at REAL NOT NULL
);
CREATE TABLE IF NOT EXISTS activity (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    ts REAL NOT NULL,
    agent TEXT NOT NULL,
    message TEXT NOT NULL,
    file_id TEXT
);
CREATE TABLE IF NOT EXISTS expenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    date TEXT NOT NULL,
    vendor TEXT NOT NULL,
    amount REAL NOT NULL,
    category TEXT NOT NULL,
    source_file TEXT NOT NULL,
    file_id TEXT,
    created_at REAL NOT NULL
);
"""

POSTGRES_SCHEMA = SQLITE_SCHEMA.replace(
    "INTEGER PRIMARY KEY AUTOINCREMENT", "SERIAL PRIMARY KEY"
)


def _is_postgres() -> bool:
    return bool(config.DATABASE_URL)


@contextmanager
def get_conn():
    if _is_postgres():
        import psycopg
        from psycopg.rows import dict_row

        conn = psycopg.connect(config.DATABASE_URL, row_factory=dict_row)
    else:
        conn = sqlite3.connect(config.SQLITE_PATH)
        conn.row_factory = sqlite3.Row
    try:
        yield conn
        conn.commit()
    finally:
        conn.close()


def _execute(conn, query: str, params: tuple = ()):
    if _is_postgres():
        query = query.replace("?", "%s")
    cur = conn.cursor()
    cur.execute(query, params)
    return cur


def init_db():
    with get_conn() as conn:
        schema = POSTGRES_SCHEMA if _is_postgres() else SQLITE_SCHEMA
        if _is_postgres():
            cur = conn.cursor()
            for statement in schema.split(";"):
                if statement.strip():
                    cur.execute(statement)
        else:
            conn.executescript(schema)


def new_id() -> str:
    return uuid.uuid4().hex[:12]


def insert_file(filename, text, summary, tags, cluster, cluster_reason,
                 doc_type, source_file_ids=None, file_id=None):
    file_id = file_id or new_id()
    with get_conn() as conn:
        _execute(
            conn,
            """INSERT INTO files
               (id, filename, text, summary, tags, cluster, cluster_reason,
                doc_type, source_file_ids, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (file_id, filename, text, summary, json.dumps(tags), cluster, cluster_reason,
             doc_type, json.dumps(source_file_ids or []), time.time()),
        )
    return file_id


def list_files():
    with get_conn() as conn:
        rows = _execute(conn, "SELECT * FROM files ORDER BY created_at DESC").fetchall()
    return [row_to_file(r) for r in rows]


def get_file(file_id):
    with get_conn() as conn:
        row = _execute(conn, "SELECT * FROM files WHERE id = ?", (file_id,)).fetchone()
    return row_to_file(row) if row else None


def get_files(file_ids):
    if not file_ids:
        return []
    with get_conn() as conn:
        placeholders = ",".join("?" for _ in file_ids)
        rows = _execute(
            conn, f"SELECT * FROM files WHERE id IN ({placeholders})", tuple(file_ids)
        ).fetchall()
    by_id = {r["id"]: row_to_file(r) for r in rows}
    return [by_id[fid] for fid in file_ids if fid in by_id]


def existing_clusters():
    with get_conn() as conn:
        rows = _execute(
            conn, "SELECT DISTINCT cluster FROM files WHERE cluster != 'Unsorted'"
        ).fetchall()
    return [r["cluster"] for r in rows]


def row_to_file(row):
    d = dict(row)
    d["tags"] = json.loads(d["tags"])
    d["source_file_ids"] = json.loads(d["source_file_ids"])
    return d


def log_activity(agent, message, file_id=None):
    with get_conn() as conn:
        _execute(
            conn,
            "INSERT INTO activity (ts, agent, message, file_id) VALUES (?, ?, ?, ?)",
            (time.time(), agent, message, file_id),
        )


def list_activity(limit=200):
    with get_conn() as conn:
        rows = _execute(
            conn, "SELECT * FROM activity ORDER BY id DESC LIMIT ?", (limit,)
        ).fetchall()
    return [dict(r) for r in rows]


def insert_expense(date, vendor, amount, category, source_file, file_id):
    with get_conn() as conn:
        _execute(
            conn,
            """INSERT INTO expenses (date, vendor, amount, category, source_file, file_id, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?)""",
            (date, vendor, amount, category, source_file, file_id, time.time()),
        )


def list_expenses():
    with get_conn() as conn:
        rows = _execute(
            conn, "SELECT * FROM expenses ORDER BY created_at DESC"
        ).fetchall()
    return [dict(r) for r in rows]


def delete_file(file_id):
    with get_conn() as conn:
        _execute(conn, "DELETE FROM files WHERE id = ?", (file_id,))


def delete_all_files():
    with get_conn() as conn:
        _execute(conn, "DELETE FROM files")


def storage_stats():
    with get_conn() as conn:
        row = _execute(
            conn,
            "SELECT COUNT(*) AS file_count, COALESCE(SUM(LENGTH(text)), 0) AS used_chars FROM files",
        ).fetchone()
    return {"file_count": row["file_count"], "used_bytes": int(row["used_chars"])}
