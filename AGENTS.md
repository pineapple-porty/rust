# Project guidance

This repository is being developed as an esolang project. Follow these guidelines for every change:

## Branch workflow

- Do not develop features directly on `main`.
- Start each feature on its own branch, based on the latest `main`. Use the naming pattern `feature/<short-description>`, for example `feature/lexer`.
- Keep all implementation, tests, and related documentation for that feature on its feature branch.
- Consider a feature finished only when its implementation is complete, relevant tests pass, and its documentation is updated where needed.
- Integrate a feature into `main` only after it is finished, using the repository's normal review and merge process. Do not use `main` as a work-in-progress branch.
- For follow-up work on an unfinished feature, continue on that feature's branch rather than starting a competing branch for the same work.

## Project structure

See [docs/layout.md](docs/layout.md) for the current repository and Rust crate layout. Keep that document up to date when adding or moving significant project components.
