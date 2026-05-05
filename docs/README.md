# Lukas docs

Status: Accepted

This directory separates current documentation by lifecycle:

- `product/`: user-facing product behavior and use cases.
- `architecture/`: accepted technical direction and domain boundaries.
- `engineering/`: implementation standards for the current repository.
- `planning/`: open questions, backlog items and cleanup notes.

Use active docs for current decisions. Archive files may contain old wording,
superseded assumptions or unresolved prompts and should not be treated as
current guidance.

## Current starting points

- Product scope: `product/overview.md`
- Use cases: `product/use-cases.md`
- Architecture overview: `architecture/overview.md`
- Engineering workflow: `engineering/dev-workflow.md`
- Open questions: `planning/open-questions.md`

## Documentation rules

Each active section should make its lifecycle explicit with one of these states:

- `Status: Accepted`
- `Status: Planned`
- `Status: Open Question`
- `Status: Deferred`

Accepted docs should not contain unresolved planning prompts. If a decision is
not settled, move it to `planning/open-questions.md` or `planning/backlog.md`.
