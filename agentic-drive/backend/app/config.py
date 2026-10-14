import os
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
SEED_DIR = BASE_DIR / "seed_data"

ON_VERCEL = bool(os.environ.get("VERCEL"))

# Accept either name: DATABASE_URL is the generic convention, POSTGRES_URL is what
# Vercel's own Postgres integration sets automatically when added to a project.
DATABASE_URL = os.environ.get("DATABASE_URL", "").strip() or os.environ.get("POSTGRES_URL", "").strip()

# SQLite fallback path. /tmp is the only writable directory on Vercel (and is wiped
# between cold starts — fine for a quick demo, but set DATABASE_URL for real persistence).
SQLITE_PATH = Path("/tmp/drive.db") if ON_VERCEL else BASE_DIR / "data" / "drive.db"
if not ON_VERCEL:
    SQLITE_PATH.parent.mkdir(parents=True, exist_ok=True)
