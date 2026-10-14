import csv
import io
from collections import defaultdict

from fastapi import FastAPI, HTTPException, UploadFile
from fastapi.responses import Response
from fastapi.staticfiles import StaticFiles
from pydantic import BaseModel

from . import config, db, storage
from .agents import orchestrator, organizer, workflow

app = FastAPI(title="Agentic Drive")


@app.on_event("startup")
def on_startup():
    db.init_db()


def _public_file(f: dict) -> dict:
    return {
        "id": f["id"],
        "filename": f["filename"],
        "summary": f["summary"],
        "tags": f["tags"],
        "cluster": f["cluster"],
        "cluster_reason": f["cluster_reason"],
        "doc_type": f["doc_type"],
        "source_file_ids": f["source_file_ids"],
        "preview": (f["text"] or "")[:300],
        "created_at": f["created_at"],
    }


def _ingest_bytes(filename: str, content: bytes) -> dict:
    file_id = db.new_id()
    text = storage.extract_text(filename, content)

    decision = organizer.organize(filename, text)
    stored_id = db.insert_file(
        filename=filename,
        text=text,
        summary=decision["summary"],
        tags=decision["tags"],
        cluster=decision["cluster"],
        cluster_reason=decision["cluster_reason"],
        doc_type=decision["doc_type"],
        file_id=file_id,
    )
    db.log_activity(
        "Organizer Agent",
        f"Filed '{filename}' under '{decision['cluster']}' with tags "
        f"[{', '.join(decision['tags'])}]. {decision['cluster_reason']}",
        file_id=stored_id,
    )
    workflow.maybe_trigger(stored_id, filename, text, decision["doc_type"])
    return db.get_file(stored_id)


@app.post("/api/files/upload")
async def upload_file(file: UploadFile):
    content = await file.read()
    if not content:
        raise HTTPException(400, "Empty file")
    record = _ingest_bytes(file.filename, content)
    return _public_file(record)


@app.post("/api/seed")
def seed():
    if not config.SEED_DIR.exists():
        return {"ingested": []}
    existing_names = {f["filename"] for f in db.list_files()}
    ingested = []
    for path in sorted(config.SEED_DIR.iterdir()):
        if not path.is_file() or path.name in existing_names:
            continue
        record = _ingest_bytes(path.name, path.read_bytes())
        ingested.append(_public_file(record))
    return {"ingested": ingested}


@app.get("/api/files")
def list_files():
    return [_public_file(f) for f in db.list_files()]


@app.get("/api/files/{file_id}")
def get_file(file_id: str):
    f = db.get_file(file_id)
    if not f:
        raise HTTPException(404, "File not found")
    return {**_public_file(f), "text": f["text"]}


@app.get("/api/clusters")
def list_clusters():
    grouped = defaultdict(list)
    for f in db.list_files():
        grouped[f["cluster"]].append(_public_file(f))
    return [{"cluster": name, "files": files} for name, files in sorted(grouped.items())]


@app.delete("/api/files/{file_id}")
def delete_file(file_id: str):
    f = db.get_file(file_id)
    if not f:
        raise HTTPException(404, "File not found")
    db.delete_file(file_id)
    db.log_activity("You", f"Deleted '{f['filename']}'.")
    return {"deleted": file_id}


class BulkDeleteRequest(BaseModel):
    file_ids: list[str]


@app.post("/api/files/delete-bulk")
def delete_files_bulk(req: BulkDeleteRequest):
    deleted = []
    for file_id in req.file_ids:
        f = db.get_file(file_id)
        if f:
            db.delete_file(file_id)
            deleted.append(f["filename"])
    if deleted:
        db.log_activity("You", f"Deleted {len(deleted)} file(s): {', '.join(deleted)}.")
    return {"deleted": deleted}


@app.delete("/api/files")
def reset_drive():
    count = len(db.list_files())
    db.delete_all_files()
    if count:
        db.log_activity("You", f"Reset the drive — deleted all {count} file(s).")
    return {"deleted_count": count}


@app.get("/api/storage")
def storage_info():
    stats = db.storage_stats()
    quota_bytes = 16 * 1024 ** 3  # cosmetic quota, matching Drive's free-tier scale
    return {**stats, "quota_bytes": quota_bytes}


class AskRequest(BaseModel):
    message: str
    selected_file_ids: list[str] = []


@app.post("/api/ask")
def ask(req: AskRequest):
    if not req.message.strip():
        raise HTTPException(400, "message is required")
    return orchestrator.handle(req.message, req.selected_file_ids)


@app.get("/api/activity")
def activity():
    return db.list_activity()


@app.get("/api/expenses")
def expenses():
    return workflow.read_expenses()


@app.get("/api/expenses.csv")
def expenses_csv():
    rows = workflow.read_expenses()
    buf = io.StringIO()
    writer = csv.DictWriter(buf, fieldnames=["date", "vendor", "amount", "category", "source_file", "file_id"])
    writer.writeheader()
    for r in rows:
        writer.writerow({k: r[k] for k in writer.fieldnames})
    return Response(
        content=buf.getvalue(),
        media_type="text/csv",
        headers={"Content-Disposition": "attachment; filename=expenses.csv"},
    )


@app.get("/api/status")
def status():
    return {"db": "postgres" if config.DATABASE_URL else "sqlite"}


# Local dev only — on Vercel, static files at the project root are served directly by
# the platform's CDN, not through this function.
FRONTEND_DIR = config.BASE_DIR.parent
if not config.ON_VERCEL and (FRONTEND_DIR / "index.html").exists():
    app.mount("/", StaticFiles(directory=str(FRONTEND_DIR), html=True), name="frontend")
