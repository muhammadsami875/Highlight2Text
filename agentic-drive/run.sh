#!/usr/bin/env bash
# Sets up a virtualenv (first run only) and starts the Agentic Drive server.
#
# Installs from the root requirements.txt (the same one Vercel's function uses) plus
# uvicorn for local serving, instead of keeping a second backend/requirements.txt —
# a second requirements.txt in the repo makes Vercel's import scanner think there are
# two separate Python apps to deploy and push you into its "Services" (multi-app
# monorepo) flow, which this project doesn't need.
set -euo pipefail
cd "$(dirname "$0")"

if [ ! -d ".venv" ]; then
  python3 -m venv .venv
fi
source .venv/bin/activate
pip install --quiet -r requirements.txt uvicorn==0.30.6

echo "Starting Agentic Drive on http://localhost:8000"
echo "(Set ANTHROPIC_API_KEY in your shell before running this to enable live agent mode.)"
exec uvicorn app.main:app --host 0.0.0.0 --port 8000 --app-dir backend
