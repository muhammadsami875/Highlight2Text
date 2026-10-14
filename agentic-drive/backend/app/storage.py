"""Text extraction from uploaded bytes.

Deliberately does not write anything to local disk: this app is designed to run as a
Vercel serverless function, where the filesystem is read-only outside of /tmp and
nothing written there survives past the current invocation. Every agent operates on
the extracted text stored in the database, not on a file path, so no disk persistence
is needed at all.
"""


def extract_text(filename: str, content: bytes) -> str:
    suffix = filename.lower().rsplit(".", 1)[-1] if "." in filename else ""
    if suffix == "pdf":
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
