# TokenTruth

Independent verification of AI coding agent token-savings claims — computed from raw session transcripts, not estimates, not marketing numbers.

You installed five tools that claim to save tokens. How much did they *actually* save in your sessions, calculated from logs that really happened?

TokenTruth reads Claude Code's own transcript files (`~/.claude/projects/**/*.jsonl`) and reconstructs real token usage per category: input, output, cache write, cache read. Everything runs locally — no network calls, read-only, your data never leaves your machine.

## Why

Compression tools (Caveman, RTK, Ponytail, Headroom, ...) mostly attack output tokens. Input tokens are usually the bulk of the bill (agentic sessions run roughly 20-25:1 input:output). Marketing numbers are self-reported and rarely audited. An independent JetBrains benchmark found Caveman's claimed 65% savings landed closer to 8.5% in a non-cherry-picked, output-only measurement.

TokenTruth is not a compression tool. It's the audit layer that sits on top of them.

## Install

As a standalone CLI:

```bash
cargo install --git https://github.com/arsyadal/tokentruth
```

Or from a clone:

```bash
git clone <repo-url> tokentruth
cd tokentruth
cargo install --path .
```

As a Claude Code plugin (adds `/tokentruth-analyze`, `/tokentruth-compare`, `/tokentruth-cost`, and lets Claude reach for the CLI on its own when you ask about token usage or want to check a savings claim — still shells out to the same binary above, install that first):

```
/plugin marketplace add arsyadal/tokentruth
/plugin install tokentruth@tokentruth
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

### Sample output

```
$ tokentruth analyze
Session 53175772-1b46-418f-aec6-549a58265fdd  (2026-07-20 23:32:27 UTC -> 2026-07-21 02:47:22 UTC)
Model: claude-sonnet-5
Turns: 268

┌─────────────┬──────────┐
│ Category    ┆ Tokens   │
╞═════════════╪══════════╡
│ Input       ┆ 290      │
├╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤
│ Output      ┆ 103454   │
├╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤
│ Cache write ┆ 611840   │
├╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤
│ Cache read  ┆ 13467392 │
├╌╌╌╌╌╌╌╌╌╌╌╌╌┼╌╌╌╌╌╌╌╌╌╌┤
│ Total       ┆ 14182976 │
└─────────────┴──────────┘
```

### Finding a session UUID

`analyze`/`cost`/`export` default to the most-recently-modified session in the current project, so most of the time you don't need a UUID at all. When you do (e.g. picking a specific past session for `compare`), transcripts live at:

```bash
ls ~/.claude/projects/<cwd-with-slashes-replaced-by-dashes>/*.jsonl
# e.g. for /Users/you/myproject:
ls ~/.claude/projects/-Users-you-myproject/*.jsonl
```

Each `.jsonl` filename (minus the extension) is the session UUID.

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

## RTK, Ponytail, Headroom — tested 2026-07-21

Ran the remaining tools claimed as "token compressors" in this environment. Raw commands, no cherry-picking.

### All 4, side by side

| Tool | What it actually compresses | Independently measured | Tool's own claim | Verdict |
|---|---|---|---|---|
| **Caveman** | Assistant output prose | Session A/B: total tokens **+116%** (fixed hook/memory overhead swamped output savings on a short task) | ~65% output savings | Real on output alone, net-negative in practice on light sessions |
| **RTK** | CLI tool-result bytes (filtered commands only) | Per-command bytes: **-16% to -86%** on filtered commands, **0%** on unfiltered passthrough | "Automatic hook, 0 tokens overhead," 99.6% dashboard | Filter savings real; "automatic hook" false (none installed); dashboard math self-inconsistent |
| **Ponytail** | Code line count (not tokens, by design) | Independent paired trial via `tokentruth compare`: total tokens **-59.3%** (mechanism: fewer wasted tool-call turns, not shorter code) — bigger effect than the author's own claimed -2% to +3.7% | No token-savings claim made | Under-sold by its own author; real and larger effect once independently measured |
| **Headroom** | LLM API request/response payload via proxy | Two independent paired trials via `tokentruth compare` (1-turn and 6-file multi-turn): total tokens **+91.1%** and **+47.7%**, cost roughly doubled both times — proxy hop breaks Anthropic's prompt cache regardless of session length | Context optimization / token savings dashboard | Measured worse than doing nothing, twice; tool's own `perf` log agrees (0.0% reduction) both times |

Every tool here either overstates its own number or ships a dashboard that doesn't reconcile with the transcripts underneath it. That gap is the whole reason this repo exists.

### RTK — real, but overstated

Byte-for-byte, `rtk`-wrapped commands vs raw, same repo, same moment:

| Command                  | Raw (bytes) | rtk (bytes) | Reduction |
|---------------------------|------------:|------------:|----------:|
| `git status`               | 100         | 49          | -51%      |
| `git log -5`                | 990         | 633         | -36%      |
| `ls -la`                   | 694         | 99          | -86%      |
| `find src -name '*.rs'`    | 80          | 67          | -16%      |
| `git show HEAD --stat`     | 585         | 585         | 0% (no filter — passthrough) |

Filtering is real for commands RTK has a dedicated filter for; unfiltered commands just pass through unchanged.

Two problems with RTK's own claims:

1. **"Automatically rewritten by the Claude Code hook... transparent, 0 tokens overhead"** (RTK.md) is false in this install. `rtk gain` itself prints `[warn] No hook installed — run 'rtk init -g' for automatic token savings`, confirming the earlier finding in this README: there is no `PreToolUse` hook in `~/.claude/settings.json`. Savings only happen when the model remembers to type `rtk <cmd>` from CLAUDE.md instructions — not mechanically enforced.
2. **`rtk gain`'s headline number doesn't reconcile with its own breakdown.** Global dashboard claims 179.7M/180.5M tokens saved (99.6%). But the single largest contributor, `rtk grep` (319 calls, "178.5M saved"), is listed at an **average of 15.3% per call** in the same table — arithmetically inconsistent with owning 99% of a 99.6%-savings total. Treat RTK's self-reported aggregate as marketing, not audit-grade; the per-command byte tests above are the trustworthy number.

### Ponytail — independently retested, real, larger effect than the author's own number

`/plugin marketplace add DietrichGebert/ponytail` only registers the marketplace; the plugin is **not** in `enabledPlugins` in `~/.claude/settings.json`, so first pass just cited the author's own self-reported benchmark (`benchmarks/results/2026-06-12-caveman-vs-ponytail.md`, n=1 per task, 5 coding tasks): total tokens -2% to +3.7% vs caveman, 2.5-5.5x fewer code lines.

That's a self-reported number from the tool's own repo — same category of claim this project exists to check — so we ran an independent paired trial instead of trusting it: identical debounce-function task, once with a clean `--setting-sources ""` baseline (no plugins, no CLAUDE.md, no hooks) and once with ponytail's skill body injected via `--append-system-prompt-file` (still no CLAUDE.md/hooks — isolates ponytail alone, not ponytail-plus-whatever-else is in the user's global config). Compared via `tokentruth compare` on the real transcripts:

| Category    | Baseline | Ponytail | Delta    | Delta %  |
|-------------|---------:|---------:|---------:|---------:|
| Input       | 16       | 6        | -10      | -62.5%   |
| Output      | 2,470    | 1,374    | -1,096   | -44.4%   |
| Cache write | 19,138   | 22,005   | +2,867   | +15.0%   |
| Cache read  | 178,996  | 58,288   | -120,708 | -67.4%   |
| **Total**   | **200,620** | **81,673** | **-118,947** | **-59.3%** |

Real, and bigger than the author's own claimed 2-16%. Turn counts explain why: baseline took **8 assistant turns** and reached for `Bash`, `Read`, and `Write` (got permission-denied in the sandbox, retried, eventually fell back to printing the code inline) — ponytail's ladder took **3 turns**, tried `Write` once, fell back immediately. The final code block in both was near-identical in size. The savings mechanism here isn't shorter code text, it's fewer wasted tool-call round-trips — consistent with the skill's own "ladder is a reflex, not a research project" rule, and a stronger, more mechanistically-explained result than the author's own n=1 number.

### Headroom — real A/B run, proxy made this trial worse

Ran the same paired-prompt methodology as the caveman/RTK A/B above: identical prompt, fresh `claude -p ... --setting-sources ""` process, once direct and once with `ANTHROPIC_BASE_URL=http://127.0.0.1:8787` (Headroom's documented wrap method) — no change made to this live session's own routing.

First pass used the `usage` block from `--output-format json` (single API-call summary) and looked mild: total tokens -7.8%, cost +55.8%. Then we dogfooded `tokentruth compare` on the actual saved transcripts for both sessions — the tool this whole repo exists to build — and the real numbers are much worse and the sign on total tokens flips:

```
tokentruth compare --before ea3d1a48-... --after 3e264141-... --project <scratch-dir>
```

| Category    | Direct  | Routed via Headroom | Delta   | Delta %   |
|-------------|--------:|---------------------:|--------:|----------:|
| Input       | 194     | 25,657                | +25,463 | +13,125%  |
| Output      | 1,010   | 1,934                 | +924    | +91.5%    |
| Cache write | 25,615  | 76,734                | +51,119 | +199.6%   |
| Cache read  | 71,914  | 84,343                | +12,429 | +17.3%    |
| **Total**   | **98,733** | **188,668**         | **+89,935** | **+91.1%** |

Total tokens roughly **doubled**, not dropped. The single-call `usage` JSON undercounted the real session by ~2x because it only reflects the final API response, not the full transcript — the routed run took 7 turns (16 JSONL lines) vs the direct run's 4 turns (14 lines), and every extra turn's tool definitions, thinking, and context get resent. Lesson within the lesson: even Claude Code's own `--output-format json` summary isn't safe to cite without checking the raw transcript — exactly the gap TokenTruth exists to close.

Headroom's own tooling still confirms zero compression, independent of which number you use: post-run, `headroom perf` reports **"Tokens: 4,673 -> 4,673 (0.0% reduction)... Total saved: 0 tokens"** and flags itself **"Unstable: 1/2 requests had cache_write > 2x cache_read."** After 11 days installed, this was Headroom's first real request in this environment. Its own dashboard says zero compression happened; the transcript says cost roughly doubled.

One paired trial on a 1-turn prompt isn't a fair test of a proxy built for context optimization on real work, so we reran it on a realistic multi-turn task instead: read all 6 `.rs` files in this repo's `src/`, summarize each, list TODOs — same direct-vs-routed pairing, `tokentruth compare` on the real transcripts again:

| Category    | Direct  | Routed via Headroom | Delta   | Delta %   |
|-------------|--------:|---------------------:|--------:|----------:|
| Input       | 10      | 7,264                 | +7,254  | +72,540%  |
| Output      | 1,127   | 4,124                 | +2,997  | +265.9%   |
| Cache write | 18,086  | 75,522                | +57,436 | +317.6%   |
| Cache read  | 105,041 | 96,600                | -8,441  | -8.0%     |
| **Total**   | **124,264** | **183,510**       | **+59,246** | **+47.7%** |

Same pattern, second independent trial, longer session: cache writes balloon (+317.6%), cache reads shrink (-8%), total tokens up 47.7%. `headroom perf` afterward: **"9,862 -> 9,862 (0.0% reduction)... Total saved: 0 tokens"**, still flagging cache instability (2/4 requests). Two for two — this isn't a short-session artifact, the proxy hop consistently breaks Anthropic's prompt cache regardless of session length in this setup.

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
