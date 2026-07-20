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
