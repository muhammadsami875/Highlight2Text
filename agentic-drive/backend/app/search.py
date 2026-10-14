"""Lightweight semantic-ish retrieval: TF-IDF over paragraph chunks + cosine similarity.

This is the first-pass retriever. The Retrieval Agent (agents/retrieval.py) takes the
top chunks this returns and, in live mode, has Claude read them and synthesize a direct
answer with citations — the combination is what makes retrieval "semantic" in practice
without requiring a hosted vector DB or an embeddings API key for the prototype to run.
"""
from sklearn.feature_extraction.text import TfidfVectorizer
from sklearn.metrics.pairwise import cosine_similarity

from . import db

CHUNK_SIZE = 700


def _chunk(text: str):
    paragraphs = [p.strip() for p in text.split("\n\n") if p.strip()]
    if not paragraphs:
        paragraphs = [text.strip()] if text.strip() else []
    chunks, buf = [], ""
    for p in paragraphs:
        if len(buf) + len(p) + 2 <= CHUNK_SIZE:
            buf = f"{buf}\n\n{p}" if buf else p
        else:
            if buf:
                chunks.append(buf)
            buf = p
    if buf:
        chunks.append(buf)
    return chunks or [text[:CHUNK_SIZE]]


def _all_chunks():
    chunks = []
    for f in db.list_files():
        for i, c in enumerate(_chunk(f["text"])):
            chunks.append({"file_id": f["id"], "filename": f["filename"], "chunk_index": i, "text": c})
    return chunks


def search(query: str, top_k: int = 6):
    chunks = _all_chunks()
    if not chunks:
        return []
    corpus = [c["text"] for c in chunks] + [query]
    vectorizer = TfidfVectorizer(stop_words="english")
    try:
        matrix = vectorizer.fit_transform(corpus)
    except ValueError:
        return []
    query_vec = matrix[-1]
    doc_matrix = matrix[:-1]
    sims = cosine_similarity(query_vec, doc_matrix)[0]
    ranked = sorted(zip(chunks, sims), key=lambda x: x[1], reverse=True)
    results = [{**c, "score": float(s)} for c, s in ranked[:top_k] if s > 0]
    return results
