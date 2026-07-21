---
name: tokentruth
description: >
  Audit real token usage and cost from Claude Code session transcripts.
  Use when the user asks how many tokens a session used, what a session
  cost, wants to verify or audit a token-savings/compression claim (from
  caveman, rtk, ponytail, headroom, or any other tool/hook/skill), or
  wants to compare token usage before vs after enabling something. Also
  use when the user says "tokentruth", "audit tokens", "real token
  usage", or asks to check a self-reported savings dashboard against
  what actually happened. Do NOT use for estimating tokens before a
  session runs, or for anything outside Claude Code transcripts.
license: MIT
---

# TokenTruth

Computes token/cost usage from Claude Code's own JSONL transcripts
(`~/.claude/projects/**/*.jsonl`) — the ground truth for what a session
actually cost, independent of any tool's self-reported dashboard.

## Before running anything

Check the binary exists: `which tokentruth`. If it's missing, tell the
user to install it and stop — do not guess at numbers or substitute
your own estimate:

```bash
cargo install --git https://github.com/arsyadal/tokentruth
```

## Commands

- `tokentruth analyze [--session <uuid>] [--project <path>]` — token
  breakdown (input/output/cache write/cache read) for one session.
  Defaults to the most recently modified session in the current
  project if `--session` is omitted.
- `tokentruth compare --before <uuid> --after <uuid> [--project <path>]`
  — delta table between two sessions. This is the right command for
  "did X actually save tokens" — run the same task twice (once with
  the thing being tested, once without) and compare the two real
  session transcripts, not two self-reported summaries.
- `tokentruth cost [--session <uuid>] [--models id1,id2] [--project <path>]`
  — USD estimate across one or more models. Defaults to the session's
  own model if `--models` is omitted.
- `tokentruth export [--session <uuid>] [--format json|csv] [--project <path>]`
  — raw per-turn data.

Session UUIDs are `.jsonl` filenames (minus extension) under
`~/.claude/projects/<cwd-with-slashes-as-dashes>/`.

## Rules

- Report the numbers the tool prints, as-is. Don't round, don't
  reinterpret, don't average across categories that mean different
  things (input/output/cache-write/cache-read all bill at different
  rates — never collapse them into one "tokens" figure without saying
  which).
- `--output-format json`'s single-call `usage` field from `claude -p`
  undercounts a multi-turn session — it's the last API response, not
  the full transcript. Prefer `tokentruth analyze`/`compare` on the
  saved transcript file over any tool's inline usage summary.
- Transcripts are subject to Claude Code's ~30-day retention by default
  (`cleanupPeriodDays` in `~/.claude/settings.json`) — a session that's
  gone can't be audited. Say so if a lookup fails, don't fabricate a
  number to fill the gap.
- This skill audits usage; it does not produce savings. Never present
  installing or running TokenTruth itself as a way to reduce tokens.
