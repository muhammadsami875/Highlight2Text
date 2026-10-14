"""Lightweight semantic-ish retrieval: pure-Python TF-IDF over paragraph chunks + cosine
similarity — no numpy/scipy/sklearn.

Why pure Python: this runs inside a Vercel serverless function, where every extra
dependency adds to cold-start time and counts against the function's bundle size limit.
This is the first-pass retriever; the Retrieval Agent (agents/retrieval.py) takes the
top chunks this returns and, in live mode, has Claude read them and synthesize a direct
answer with citations — the combination is what makes retrieval "semantic" in practice
without a hosted vector DB or an embeddings API key.
"""
import math
import re
from collections import Counter

from . import db

CHUNK_SIZE = 700
STOPWORDS = {
    "the", "a", "an", "and", "or", "but", "for", "nor", "on", "at", "to", "from",
    "by", "of", "in", "with", "is", "are", "was", "were", "be", "been", "being",
    "this", "that", "these", "those", "it", "its", "as", "if", "then", "than",
    "so", "not", "no", "we", "you", "they", "i", "he", "she", "do", "does", "did",
}


def _tokenize(text: str) -> list[str]:
    words = re.findall(r"[a-z0-9]+", text.lower())
    return [w for w in words if w not in STOPWORDS and len(w) > 1]


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


def _tfidf_vectors(docs: list[list[str]]) -> list[dict[str, float]]:
    """docs: tokenized documents (last one is the query). Returns a TF-IDF weight dict per doc."""
    n_docs = len(docs)
    doc_freq = Counter()
    for tokens in docs:
        for term in set(tokens):
            doc_freq[term] += 1

    vectors = []
    for tokens in docs:
        term_counts = Counter(tokens)
        total = sum(term_counts.values()) or 1
        vec = {}
        for term, count in term_counts.items():
            tf = count / total
            idf = math.log((n_docs + 1) / (doc_freq[term] + 1)) + 1
            vec[term] = tf * idf
        vectors.append(vec)
    return vectors


def _cosine(a: dict[str, float], b: dict[str, float]) -> float:
    common = set(a) & set(b)
    if not common:
        return 0.0
    dot = sum(a[t] * b[t] for t in common)
    norm_a = math.sqrt(sum(v * v for v in a.values())) or 1.0
    norm_b = math.sqrt(sum(v * v for v in b.values())) or 1.0
    return dot / (norm_a * norm_b)


def search(query: str, top_k: int = 6):
    chunks = _all_chunks()
    if not chunks:
        return []

    docs = [_tokenize(c["text"]) for c in chunks] + [_tokenize(query)]
    vectors = _tfidf_vectors(docs)
    query_vec = vectors[-1]
    doc_vecs = vectors[:-1]

    scored = [(c, _cosine(query_vec, v)) for c, v in zip(chunks, doc_vecs)]
    scored.sort(key=lambda x: x[1], reverse=True)
    return [{**c, "score": s} for c, s in scored[:top_k] if s > 0]
