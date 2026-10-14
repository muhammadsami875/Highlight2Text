import json
import sqlite3
import time
import uuid
from contextlib import contextmanager

from . import config

SCHEMA = """
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY,
    filename TEXT NOT NULL,
    stored_path TEXT NOT NULL,
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
"""


@contextmanager
def get_conn():
    conn = sqlite3.connect(config.DB_PATH)
    conn.row_factory = sqlite3.Row
    try:
        yield conn
        conn.commit()
    finally:
        conn.close()


def init_db():
    with get_conn() as conn:
        conn.executescript(SCHEMA)


def new_id() -> str:
    return uuid.uuid4().hex[:12]


def insert_file(filename, stored_path, text, summary, tags, cluster, cluster_reason,
                 doc_type, source_file_ids=None, file_id=None):
    file_id = file_id or new_id()
    with get_conn() as conn:
        conn.execute(
            """INSERT INTO files
               (id, filename, stored_path, text, summary, tags, cluster, cluster_reason,
                doc_type, source_file_ids, created_at)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (file_id, filename, str(stored_path), text, summary, json.dumps(tags),
             cluster, cluster_reason, doc_type, json.dumps(source_file_ids or []), time.time()),
        )
    return file_id


def list_files():
    with get_conn() as conn:
        rows = conn.execute("SELECT * FROM files ORDER BY created_at DESC").fetchall()
    return [row_to_file(r) for r in rows]


def get_file(file_id):
    with get_conn() as conn:
        row = conn.execute("SELECT * FROM files WHERE id = ?", (file_id,)).fetchone()
    return row_to_file(row) if row else None


def get_files(file_ids):
    if not file_ids:
        return []
    with get_conn() as conn:
        placeholders = ",".join("?" for _ in file_ids)
        rows = conn.execute(
            f"SELECT * FROM files WHERE id IN ({placeholders})", file_ids
        ).fetchall()
    by_id = {r["id"]: row_to_file(r) for r in rows}
    return [by_id[fid] for fid in file_ids if fid in by_id]


def existing_clusters():
    with get_conn() as conn:
        rows = conn.execute(
            "SELECT DISTINCT cluster FROM files WHERE cluster != 'Unsorted'"
        ).fetchall()
    return [r["cluster"] for r in rows]


def row_to_file(row):
    d = dict(row)
    d["tags"] = json.loads(d["tags"])
    d["source_file_ids"] = json.loads(d["source_file_ids"])
    return d


def log_activity(agent, message, file_id=None):
    with get_conn() as conn:
        conn.execute(
            "INSERT INTO activity (ts, agent, message, file_id) VALUES (?, ?, ?, ?)",
            (time.time(), agent, message, file_id),
        )


def list_activity(limit=200):
    with get_conn() as conn:
        rows = conn.execute(
            "SELECT * FROM activity ORDER BY id DESC LIMIT ?", (limit,)
        ).fetchall()
    return [dict(r) for r in rows]
