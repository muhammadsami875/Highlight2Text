# OCR accuracy

Per-fixture CER / WER, measured by `scripts/eval-ocr.ps1` against the
corpus described in [Phase 19](PHASE_19_TESTING.md).

| Class                | Language | CER | WER | n | Notes |
|---                   |---       |---  |---  |---|---    |
| Clean printed        | eng      |     |     |   |       |
| Low-light            | eng      |     |     |   |       |
| Skewed               | eng      |     |     |   |       |
| Perspective          | eng      |     |     |   |       |
| Shadow               | eng      |     |     |   |       |
| Colored background   | eng      |     |     |   |       |
| Low-res              | eng      |     |     |   |       |
| High-res             | eng      |     |     |   |       |
| Multi-column         | eng      |     |     |   |       |
| Table                | eng      |     |     |   |       |
| Printed Urdu         | urd      |     |     |   |       |
| Printed Arabic       | ara      |     |     |   |       |
| EN + UR mixed        | eng+urd  |     |     |   |       |
| EN + AR mixed        | eng+ara  |     |     |   |       |
| Photographed doc     | eng      |     |     |   |       |
| PDF-rendered page    | eng      |     |     |   |       |

Rules of the house:

1. **No global "accuracy %" claim** — only per-class rows after measurement.
2. CER = Levenshtein / len(reference); WER = word-level same.
3. Measurement image is whatever the pipeline would feed OCR
   (`enhanced > warped > original`), not a hand-tuned preprocessor.
4. Each release notes the delta vs. the previous release.
