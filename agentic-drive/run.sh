#!/usr/bin/env bash
# Sets up a virtualenv (first run only) and starts the Agentic Drive server.
set -euo pipefail
cd "$(dirname "$0")/backend"

if [ ! -d ".venv" ]; then
  python3 -m venv .venv
fi
source .venv/bin/activate
pip install --quiet -r requirements.txt

echo "Starting Agentic Drive on http://localhost:8000"
echo "(Set ANTHROPIC_API_KEY in your shell before running this to enable live agent mode.)"
exec uvicorn app.main:app --host 0.0.0.0 --port 8000 --app-dir .
