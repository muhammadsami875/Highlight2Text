"""Vercel entrypoint: re-exports the FastAPI app for Vercel's Python runtime.

vercel.json rewrites every /api/* request to this file; the FastAPI app's own routes
(all already prefixed with /api/...) handle the actual dispatch from there.
"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent / "backend"))

from app.main import app  # noqa: E402,F401
