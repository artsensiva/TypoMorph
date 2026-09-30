# TypoMorph working rules

- Use clear, concise English for project communication, code, comments,
  documentation, tests, issues, commits, and release notes. Preserve necessary
  multilingual product strings and test fixtures.
- Continue existing work; do not restart discovery without a concrete reason.
- Source-of-truth priority:
  1. Explicit owner-approved conversation decisions.
  2. Current local workspace.
  3. Specifications and decision documents.
  4. Implementation and tests.
  5. README and roadmap.
  6. Historical documents such as PROMPT.md.
- Identify contradictions explicitly. Never silently override approved decisions.
  Existing implementation does not redefine approved product requirements.
- The owner decides product, monetization, scope, privacy, and architecture.
  Present realistic options, trade-offs, and a recommendation; ask one focused
  decision question at a time.
- Before significant implementation changes, explain purpose, affected
  components, and risks, then obtain approval. Continue small steps already
  covered by an approved plan without repeated approval.
- Obtain explicit authorization before commits, pushes, publication, or deployment.
- Prefer simple, maintainable implementations. Avoid speculative features and
  unrelated refactors. Preserve unrelated user changes.
- Run relevant tests and report their scope and limitations. Distinguish
  synthetic fixtures, known-gap reproductions, and real application evidence.
  Never present simulated behavior as working production functionality.
- Process typed text locally in bounded transient memory. Release builds must
  not persist or transmit typed content. Protected or unknown fields must not
  be analyzed or corrected. Never bypass safety gates for compatibility.
- Use controlled synthetic input for necessary development diagnostics.
  Coordinate live tests; do not capture personal typing.
- Do not translate existing documentation without approval. Avoid stylistic
  translation diffs.
- After meaningful milestones or approved decisions, synchronize
  docs/PROJECT_STATE.md and directly affected documentation only.
- Keep project state concise, factual, and explicit about unresolved blockers.
  Prefer targeted inspection over repeatedly rereading the repository.
