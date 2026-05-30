---
name: atomic-commits
description: >-
  Creates minimal, atomic git commits using Conventional Commits—one logical
  unit per commit, scoped staging, split by concern. Use when the user asks to
  commit, save changes, split commits, or follow project commit standards.
---

# Atomic Commits

Create **several small commits**, each with **one complete logical change** and a **short Conventional Commit** subject. Prefer many atomic commits over one large commit.

This project defines commit standards in [AGENTS.md](../../AGENTS.md). Read that file first; this skill is the detailed workflow.

## When to apply

- User explicitly asks to commit (`commit`, `commit isso`, `atomic commits`, etc.).
- User asks to **re-split** commits after a messy history.
- **Do not** commit unless the user asked.

## Message format

```
type(scope): imperative subject
```

| Type | Use for |
|------|---------|
| `feat` | New behavior users/systems can use |
| `fix` | Corrects broken or missing behavior |
| `docs` | Documentation only |
| `chore` | Tooling, gitignore, readme samples, no product logic |
| `refactor` | Structure change, same behavior |
| `test` | Tests only |
| `build` | Cargo, lockfile, CI build |
| `ci` | CI config |

**Rules for the subject:**

- One line, **≤ 72 characters**, imperative mood (`add`, not `added`).
- **No period** at the end.
- **No body** unless the user asks for detail.
- `scope` is optional but preferred (`core`, `server`, `web`, `docker`).

**Examples (this repo):**

```
chore: add gitignore
feat(core): add orchestrator-core
feat(server): add orchestrator-server cli
docs: add v1 requirements and plan
chore: add readme and workflow example
```

## Atomicity rules

One commit = **one concern** that could be reverted alone without orphaning the tree.

| Commit separately | Do not mix in the same commit |
|-------------------|-------------------------------|
| `.gitignore` / editor config | Feature code |
| Each crate or major module (`orchestrator-core`, `orchestrator-server`) | Unrelated crate |
| `docs/` only | Rust source |
| README / examples | Core library |
| Lockfile with the workspace it locks | Random later fix |

**Split order (typical greenfield):**

1. `chore:` gitignore and local-only exclusions
2. `feat(core):` or `build:` workspace + core crate (+ `Cargo.lock` if new workspace)
3. `feat(server):` server/binary crate
4. `docs:` requirements, plans, ADRs
5. `chore:` readme, examples, scripts

If separation is unclear, **one commit is acceptable**—do not over-split hunks inside a file.

## Workflow

### 1. Gather context (parallel)

```bash
git status
git diff
git log --oneline -5
```

Check: secrets (`.env`, keys), local-only paths (`.claude/`, `.data/`), generated artifacts that belong in `.gitignore`.

### 2. Plan commits

List planned commits **before** staging (short table: files → message). Confirm split matches atomicity rules.

### 3. Commit sequentially

For each unit:

```bash
git add <paths-for-this-unit-only>
git commit -m "type(scope): subject"
```

- **Never** `git add .` unless the entire tree is one logical unit.
- **Never** `git commit --amend` unless user rules allow it (hook failure, your HEAD commit, not pushed).
- **Never** change `git config`, skip hooks, or force-push without explicit user request.

### 4. Verify

```bash
git status
git log --oneline
```

Working tree should be clean (or only intentional untracked files).

## Recover from mistakes

**Wrong files in a commit (not pushed):**

```bash
git reset --mixed <last-good-commit>
# Re-stage and commit in correct atomic order
```

**`index.lock` (Windows / parallel tools):**

Remove `.git/index.lock` only when no other git process is running, then retry.

## Safety

- Do not commit: `.env`, credentials, `.claude/`, `target/`, `*.db`, `node_modules/`, `.data/`.
- Do not push unless the user asks.
- Match existing repo history when present; otherwise use the format above.

## Relation to other skills

- Single commit with rich message → use `ce-commit` if installed.
- This skill → **multiple minimal atomic commits** by default when the user asks to commit work that spans layers (core, server, docs).
