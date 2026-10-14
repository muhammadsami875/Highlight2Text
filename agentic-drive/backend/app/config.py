import os
from pathlib import Path

BASE_DIR = Path(__file__).resolve().parent.parent
DATA_DIR = BASE_DIR / "data"
FILES_DIR = DATA_DIR / "files"
DB_PATH = DATA_DIR / "drive.db"
EXPENSES_CSV_PATH = DATA_DIR / "expenses.csv"
SEED_DIR = BASE_DIR / "seed_data"

FILES_DIR.mkdir(parents=True, exist_ok=True)

ANTHROPIC_API_KEY = os.environ.get("ANTHROPIC_API_KEY", "").strip()
ANTHROPIC_MODEL = os.environ.get("ANTHROPIC_MODEL", "claude-sonnet-4-5")

LIVE_MODE = bool(ANTHROPIC_API_KEY)
