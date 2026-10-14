from pathlib import Path

from . import config


def save_bytes(file_id: str, filename: str, content: bytes) -> Path:
    folder = config.FILES_DIR / file_id
    folder.mkdir(parents=True, exist_ok=True)
    path = folder / filename
    path.write_bytes(content)
    return path


def extract_text(path: Path, content: bytes) -> str:
    suffix = path.suffix.lower()
    if suffix == ".pdf":
        try:
            from pypdf import PdfReader
            import io

            reader = PdfReader(io.BytesIO(content))
            return "\n".join((page.extract_text() or "") for page in reader.pages)
        except Exception as exc:  # pragma: no cover - defensive
            return f"[could not extract PDF text: {exc}]"
    try:
        return content.decode("utf-8")
    except UnicodeDecodeError:
        return content.decode("latin-1", errors="ignore")
