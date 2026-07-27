# Implementation Plan: SQL Database Engine with B+Tree Storage (Rust)

A phased plan for a disk-backed database engine with a SQL front-end, sized for a strong portfolio project. Each phase produces something demonstrable, so the project has value even if you stop early.

**Target skill mapping:** Your C# background covers the OOP → trait translation well. Chapters 1–14 of the Rust book cover everything needed for Phases 0–2. You'll need Chapter 15 (Box, Rc, RefCell) before Phase 3 — read it when you get there, not before.

---

## Architecture Overview

```
 SQL string
     │
 ┌───▼────────┐
 │ Tokenizer  │  "SELECT * FROM users" → [Select, Star, From, Ident("users")]
 └───┬────────┘
 ┌───▼────────┐
 │ Parser     │  tokens → AST (recursive descent)
 └───┬────────┘
 ┌───▼────────┐
 │ Executor   │  AST → calls into storage layer
 └───┬────────┘
 ┌───▼────────┐
 │ B+Tree     │  insert / point lookup / range scan
 └───┬────────┘
 ┌───▼────────┐
 │ Pager      │  page cache ↔ 4KB pages on disk
 └───┬────────┘
    file.db
```

Key design decision: **the B+Tree operates on disk pages, not in-memory node structs.** Each node *is* a 4KB page addressed by page number (a `u32`). This matters for two reasons:

1. It's how real databases (SQLite, Postgres) work — that's the portfolio talking point.
2. It sidesteps the borrow checker pain of pointer-based trees. Nodes reference each other by page number (just an integer), not by `Rc<RefCell<Node>>`. No fighting the borrow checker over parent pointers.

---

## Phase 0 — REPL Shell (Week 1)

**Goal:** A running binary you can type into.

- Read-eval-print loop over stdin
- Meta-commands: `.exit`, `.tables`, `.help` (dot prefix, handled before SQL parsing — same convention as SQLite)
- Error type skeleton: define a top-level `DbError` enum early with `thiserror`, and make everything return `Result<T, DbError>`. Retrofitting error handling later is miserable.

**Rust concepts exercised:** `match`, enums, `Result`, `std::io`.

---

## Phase 1 — SQL Tokenizer + Parser (Weeks 1–3)

**Goal:** Turn SQL text into an AST. Hand-roll it — using the `sqlparser` crate would gut the most interview-discussable part of the project.

Supported subset (resist expanding this early):

```sql
CREATE TABLE users (id INTEGER PRIMARY KEY, name TEXT, age INTEGER);
INSERT INTO users VALUES (1, 'daniel', 24);
SELECT * FROM users;
SELECT name, age FROM users WHERE id = 1;
SELECT * FROM users WHERE age > 20;
```

Steps:

1. **Tokenizer:** iterator over chars → `Vec<Token>`. Token enum: keywords, identifiers, string/int literals, punctuation. ~150 lines.
2. **AST types:** `Statement` enum with `CreateTable`, `Insert`, `Select` variants holding structs.
3. **Recursive descent parser:** one function per grammar rule (`parse_statement`, `parse_select`, `parse_where`). Peekable token iterator. ~300 lines.
4. **Tests:** parser tests are pure functions — easy wins. Write them as you go.

Coming from C#: the `Token`/`Statement` enums with payload data are what you'd model as a class hierarchy with pattern matching in C#. Rust enums + `match` do this natively and the compiler forces exhaustiveness — lean on that.

**Checkpoint demo:** REPL prints the parsed AST via `#[derive(Debug)]`.

---

## Phase 2 — Pager + Row Storage (Weeks 3–5)

**Goal:** Rows persist to disk and survive restart. No B+Tree yet — append-only heap of pages first.

1. **Page format:** fixed 4096-byte pages. `Page` = `[u8; 4096]` wrapper.
2. **Pager struct:** owns the `File`, exposes `get_page(page_num) -> &mut Page`, tracks dirty pages, flushes on demand. Start with a simple `HashMap<u32, Page>` cache; LRU eviction is a later refinement.
3. **Row serialization:** manual little-endian encoding (fixed-width int, length-prefixed text). Doing this by hand instead of reaching for `serde`/`bincode` is deliberate — byte-level format control is the point, and it's what you'll discuss in interviews.
4. **Table = pages of rows** with a small page header (row count, free-space offset). Slotted pages if you're feeling ambitious, simple fixed-slot layout if not.
5. Wire up: `INSERT` appends a row, `SELECT` scans all pages linearly.

**Rust concepts exercised:** ownership of buffers, slices, byte manipulation (`u32::to_le_bytes` / `from_le_bytes`), lifetimes start appearing here.

**Checkpoint demo:** insert rows, quit, restart, `SELECT` returns them. This alone is already a decent mini-project.

---

## Phase 3 — B+Tree (Weeks 5–9, the core)

**Goal:** Replace linear scan with a B+Tree keyed on the primary key.

Brush-up note (this is where "midlevel B-Tree understanding" gets tested): in a **B+Tree**, internal nodes hold only keys and child page numbers; **all rows live in leaf nodes**, and leaves are chained left-to-right via a `next_leaf` page pointer. That chain is what makes `WHERE age > 20`-style range scans cheap.

Build order:

1. **Node page layout.** Header: node type (leaf/internal), key count, parent page num. Leaf body: sorted (key, row) cells. Internal body: sorted (key, child page num) pairs plus one rightmost child pointer.
2. **Search:** descend from root, binary search within each node, arrive at a leaf. Recursion here is on *page numbers*, so no borrow checker drama.
3. **Insert without splits** (node has room): find leaf, shift cells, insert sorted.
4. **Leaf split:** node full → allocate new page, move upper half, push middle key into parent.
5. **Internal split + root split:** the hardest part. Root split grows tree height. Budget 2 weeks for steps 4–5 and test obsessively.
6. **Range scan:** find start leaf, walk the leaf chain.

**Testing strategy (do not skip):** property-style test — insert N keys in random order, verify an in-order traversal returns them sorted and every node respects capacity invariants. Run with N large enough to force multiple levels (~1000+ with small fake page sizes). Shrink page size to 64 bytes in tests so splits happen constantly.

**Deliberately out of scope for v1:** DELETE with node merging/rebalancing. Real databases (SQLite included) often just tombstone instead of merging. Note it in the README as a known simplification — that reads as informed judgment, not laziness.

**Checkpoint demo:** point lookup on 100k rows is instant; before/after benchmark vs. Phase 2's linear scan makes a great README graph.

---

## Phase 4 — Executor Glue + Polish (Weeks 9–11)

- Executor maps AST → storage calls: `WHERE id = x` → tree point lookup; `WHERE age > x` on a non-key column → full scan with filter (be honest about this in the README; secondary indexes are the natural "future work" item).
- Column projection (`SELECT name, age`).
- A `catalog` table (or page 0 header region) storing table schemas so `CREATE TABLE` persists.
- Formatted table output in the REPL.

---

## Phase 5 — Stretch Goals (pick one, not all)

Ranked by portfolio value per effort:

1. **Write-ahead log + crash recovery** — the strongest differentiator; demo by `kill -9`-ing mid-insert and recovering.
2. **Secondary indexes** — second B+Tree mapping column value → primary key; reuses Phase 3 code.
3. **LRU page cache with eviction** — good systems talking point, small code.
4. `EXPLAIN` output showing scan vs. index lookup.

---

## Project Hygiene (portfolio-specific)

- **Repo structure:** `src/main.rs` (REPL), `sql/` (tokenizer, parser, ast), `storage/` (pager, btree, row). Module boundaries mirror the architecture diagram.
- **README:** architecture diagram, supported SQL grammar, the benchmark graph, known limitations. A limitations section signals maturity.
- **Commit history:** commit per milestone with real messages — reviewers do look.
- **CI:** GitHub Actions running `cargo test` + `cargo clippy`. Trivial to set up, looks professional.

## Resource Policy

**Learning goal:** a thorough, marketable understanding of both the low-level language (Rust ownership, byte-level memory layout) and the low-level data structure (B+Tree mechanics). The restrictions below exist to force that understanding — the constraint is the curriculum. The test throughout: *can you explain and re-derive every line you committed?*

### Permitted — concept resources (read freely)

- **The Rust Book** (doc.rust-lang.org/book) — Ch. 15 before Phase 3, Ch. 11 before writing tests
- **Rust std library docs** (doc.rust-lang.org/std) — unlimited; reading std docs is a professional skill, not a crutch
- **CMU 15-445 lectures** (YouTube) — Buffer Pools and B+Tree lectures specifically
- ***Database Internals*, Petrov** — Part I only, as a reference while building Phase 3
- **SQLite file format documentation** (sqlite.org/fileformat2.html) — study a real page layout before designing yours
- **Compiler error output** — rustc's errors are a teaching tool; read them fully before searching anything

### Permitted with rules — implementation references

- ***Let's Build a Simple Database*** (cstack.github.io/db_tutorial) — the plan follows its arc, but it's in C. Rule: read a chapter to understand the *approach*, close the tab, implement in Rust from memory. Never transliterate the C line-by-line — the translation gap is where the Rust learning happens.
- **Web searches / Stack Overflow** — permitted for *language mechanics* ("rust convert u32 to bytes", "rust lifetime error E0502") and tooling issues. Not permitted for *algorithm logic* ("b+tree split implementation rust", "how to write sql parser") — those must come from the concept resources above plus your own derivation.

### Not permitted

- **AI code generation** for any core logic (parser, pager, B+Tree). Every line in `sql/` and `storage/` must be hand-derived. AI is acceptable only for explaining a concept you then implement independently, or reviewing code you already wrote.
- **Crates for core functionality** — no `sqlparser`, no `serde`/`bincode` for row encoding, no B-Tree or storage crates. Whitelist: `thiserror` (error boilerplate) and `criterion` (benchmarking) only, since neither replaces learning.
- **Copying from existing Rust database projects** (toydb, sled, etc.). Don't read their source until *after* your own Phase 3 works — then a compare-and-contrast makes an excellent README or blog-post section.

### Escalation path when stuck

In order: (1) re-read the relevant concept resource, (2) write the failing case out on paper — B+Tree splits especially should be drawn by hand before coded, (3) shrink the test page size and step through with `dbg!()`/a debugger, (4) only then a targeted mechanics search. If a bug survives two sessions, that's the topic you understand least — which makes it the highest-value thing in the project, and a story worth telling in interviews.

## Time Budget

~11 weeks at 8–10 hrs/week. If time-boxed harder, Phases 0–3 alone (a persistent key-value store with a SQL front-end) is still a complete, presentable project — cut Phase 5 first, then Phase 4 polish.
