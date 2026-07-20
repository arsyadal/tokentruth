# TokenTruth

Independent verification of AI coding agent token-savings claims — computed from raw session transcripts, not estimates, not marketing numbers.

You installed five tools that claim to save tokens. How much did they *actually* save in your sessions, calculated from logs that really happened?

TokenTruth reads Claude Code's own transcript files (`~/.claude/projects/**/*.jsonl`) and reconstructs real token usage per category: input, output, cache write, cache read. Everything runs locally — no network calls, read-only, your data never leaves your machine.

## Why

Compression tools (Caveman, RTK, Ponytail, Headroom, ...) mostly attack output tokens. Input tokens are usually the bulk of the bill (agentic sessions run roughly 20-25:1 input:output). Marketing numbers are self-reported and rarely audited. An independent JetBrains benchmark found Caveman's claimed 65% savings landed closer to 8.5% in a non-cherry-picked, output-only measurement.

TokenTruth is not a compression tool. It's the audit layer that sits on top of them.

## Install

```bash
git clone <repo-url> tokentruth
cd tokentruth
cargo install --path .
```

## Usage

```bash
# Analyze the most recent session in the current project
tokentruth analyze

# Analyze a specific session
tokentruth analyze --session <uuid>

# Compare two sessions (e.g. before/after enabling a compression tool)
tokentruth compare --before <session-a> --after <session-b>

# Estimate cost across models for a session
tokentruth cost --session <uuid> --models claude-sonnet-5,claude-opus-4-8

# Export raw per-turn data for further analysis
tokentruth export --session <uuid> --format json|csv
```

All commands default to the current directory's project. Pass `--project <path>` to target another project's transcripts.

## Case study (real data)

Ran `tokentruth analyze --breakdown` against a real ~4hr Claude Code session (273 turns, no compression tool active):

| Category    | Tokens     |
|-------------|-----------:|
| Input       | 347        |
| Output      | 117,574    |
| Cache write | 1,642,669  |
| Cache read  | 12,010,631 |
| **Total**   | **13,771,221** |

Estimated cost at current pricing:

| Model            | Total cost |
|------------------|-----------:|
| claude-sonnet-5  | $11.53     |
| claude-opus-4-8  | $57.64     |

Cache read tokens dominate this session (87% of total) — a compression tool that only shrinks assistant *output* text would have no measurable effect on the actual bill here. This is exactly the kind of gap TokenTruth is built to surface.

## Controlled A/B: hooks + memory ON vs OFF

Real sessions from history aren't a fair A/B (different tasks, different lengths). So we ran the *same prompt*, twice, back to back, in the same empty scratch directory, using Claude Code's own `--setting-sources` flag to toggle project/user hooks and CLAUDE.md memory on and off:

```bash
# ON: normal session — global hooks + CLAUDE.md (caveman mode + RTK instructions) active
claude -p "$PROMPT" --output-format json

# OFF: same prompt, same directory — no hooks, no CLAUDE.md memory
claude -p "$PROMPT" --setting-sources "" --output-format json
```

Prompt: *"Run git status and ls -la in the current directory, then explain in a few sentences what a binary search tree is and give a short Python example with insert and search methods."*

Then audited both resulting transcripts with `tokentruth compare`:

| Category    | OFF (baseline) | ON (hooks + CLAUDE.md) | Delta    | Delta %  |
|-------------|----------------:|------------------------:|---------:|---------:|
| Input       | 6               | 560                     | +554     | +9233%   |
| Output      | 723             | 1,954                   | +1,231   | +170%    |
| Cache write | 20,012          | 83,012                  | +63,000  | +315%    |
| Cache read  | 68,199          | 106,770                 | +38,571  | +57%     |
| **Total**   | **88,940**      | **192,296**             | **+103,356** | **+116%** |
| API cost (reported by Claude Code) | $0.086 | $0.502 | +$0.416 | +484% |

**On this lightweight, single-turn task, hooks + CLAUDE.md memory more than doubled total token usage instead of saving anything.** This matches the "honest number warning" already printed in Caveman's own README: compression that only targets assistant *output* text can be net-negative once you account for the fixed input-token cost of the instructions/hooks themselves — here, ~63k extra cache-write tokens just to load the CLAUDE.md/RTK.md context once, plus repeated `CAVEMAN MODE ACTIVE` hook injections per turn.

A second finding, independent of the token count: RTK's own docs claim commands are "automatically rewritten by the Claude Code hook" (`git status` → `rtk git status`, "transparent, 0 tokens overhead"). We checked `~/.claude/settings.json` directly — there is no `PreToolUse` hook registered anywhere that does this rewrite. Only `SessionStart` and `UserPromptSubmit` hooks exist (both belong to caveman mode). In this ON run the model called plain `git status` and `ls -la`, unrewritten. The "automatic hook" savings RTK's docs describe don't appear to be backed by an actual hook in this installation — savings depend on the model choosing to type `rtk <cmd>` itself from the CLAUDE.md instructions, not on mechanical enforcement.

**Caveats (read before citing this number):** this is one paired trial, not a statistically powered benchmark. The ON run took 9 turns vs OFF's 5 (the model split `git status`/`ls -la` into two calls in the ON run vs one combined call OFF), which inflates ON's output-token count independent of caveman styling. Reproduce it yourself — the exact commands are above, and the raw transcripts this table came from are session `9a36b799-9bcc-45ea-b6f6-06c49da19a2c` (ON) and `11e4847f-3290-4fde-a2ab-d187da8529b0` (OFF).

## Methodology

- Parses Claude Code's JSONL transcripts directly (`type: user|assistant|system`, `message.usage.{input_tokens,output_tokens,cache_creation_input_tokens,cache_read_input_tokens}`).
- Unknown fields and malformed lines are skipped, not fatal — the transcript format is internal to Claude Code and can change between releases without notice.
- Pricing is a static, bundled table (USD per million tokens), updated manually per release. No automatic network fetch.
- No sampling, no estimation, no hidden math. The full calculation is in [`src/`](src/).

## Scope

MVP targets Claude Code only. Not a live monitor ("htop for agents") — this is post-hoc audit from transcripts. Not an enterprise billing/invoice reconciliation tool. Not a compression tool itself (would create a conflict of interest with the audit function).

See [`prd.md`](prd.md) for full product spec, roadmap, and open questions.

## License

MIT
