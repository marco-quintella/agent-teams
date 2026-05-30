# Agent instructions — claude-orchestrator

## Commits

When the user asks to **commit** changes, follow **atomic, minimal commits**—not one large commit.

**Skill (full workflow):** [.cursor/skills/atomic-commits/SKILL.md](.cursor/skills/atomic-commits/SKILL.md)

Invoke that skill (or apply it directly) for every commit request.

### Quick rules

- **Format:** `type(scope): imperative subject` (Conventional Commits)
- **One commit = one logical unit** (e.g. `chore: add gitignore`, `feat(core): …`, `feat(server): …`, `docs: …`)
- **Short subject:** one line, no trailing period, ≤ 72 characters
- **Stage by path:** `git add <specific paths>` — avoid `git add .` unless everything is one unit
- **Only commit when the user asks**; never push unless asked
- **Never commit:** `.env`, secrets, `.claude/`, `target/`, `*.db`, `node_modules/`, `.data/`

### Types

| Type | Use |
|------|-----|
| `feat` | New capability |
| `fix` | Bug fix |
| `docs` | Documentation |
| `chore` | Gitignore, readme, examples |
| `test` | Tests only |
| `build` | Workspace / lockfile / Docker build |
| `refactor` | Same behavior, different structure |

## Planning and execution

- Product/requirements: `docs/brainstorms/`
- Implementation plan: `docs/plans/`
- Rust workspace: `crates/orchestrator-core`, `crates/orchestrator-server`
- UI (planned): `web/` (Svelte + Vite)

## Build (Windows)

Requires MSVC `link.exe` (Visual Studio Build Tools with C++ workload):

```bash
cargo check --workspace
cargo test -p orchestrator-core
```
