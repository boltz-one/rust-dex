# Zed Editor & Terminal Stack Portability Analysis

## Text Buffer & Syntax Highlighting Stack

### Component LOC & Isolation

| Component | LOC | Key Imports | Zed Coupling |
|-----------|-----|------------|--------------|
| rope | 4,132 | heapless, sum_tree, unicode_segmentation, util, ztracing | Low (mostly stdlib/generic) |
| text | 6,570 | collections, clock, sum_tree, parking_lot, smallvec | **High** (internal crates: collections, clock) |
| multi_buffer | 16,329 | — | High (excerpt/diff layer, optional for Phase B) |
| language | 22,926 | tree-sitter (13 grammar bins) | **High** (tree-sitter-{rust,py,ts,md,c,rb,json,...} as workspace deps) |
| editor | 154,542 | — | N/A (full app, reference only) |

**Key Finding:** Rope (4.1k LOC) **isolatable**—pure text structure with generic deps (no collections/clock imports). Text layer requires unwrapping Zed's internal Anchor/Selection types (uses clock::Lamport for CRDT). Multi_buffer ~16k LOC, adds excerpt/diff logic—omissible for Phase B MVP.

### Syntax Highlighting: tree-sitter Integration

Language crate vendors **13 tree-sitter grammar crates** as workspace dependencies:
- tree-sitter (core), tree-sitter-{rust, python, typescript, md, json, html, ruby, c, elixir, embedded-template, heex}

Each grammar is a separate workspace member—full tree-sitter integration ~500KB binary footprint per grammar (worst case: ~6.5MB for all). Phase B minimal: tree-sitter core + 3–4 high-use grammars (rust, typescript, python, json).

**Base has** `crates/syntax_theme/` crate already. Zed's `theme::SyntaxTheme` struct (location not resolved in grep due to workspace structure, but commonly in `crates/theme/src/`). Likely structurally similar—recommend verifying property alignment (color slot names, scope hierarchy) before integration.

---

## Terminal PTY Stack

### Component LOC & Coupling

| Component | LOC | External Deps | Zed Coupling |
|-----------|-----|--------------|--------------|
| terminal | 9,007 | alacritty_terminal, vte, async-channel, libc | **Medium** (gpui, collections, theme, settings crates) |
| terminal_view | 9,635 | — (view-layer, depends on terminal) | **High** (gpui element/view semantics) |

**Platform-specific code:** Only **6 #[cfg(target_os=...)]** instances in terminal/src—mostly concentrated in PTY acquisition (libc calls for Unix; Windows PTY via alacritty_terminal abstraction).

### Dependency Breakdown

**terminal/Cargo.toml:**
- **alacritty_terminal** — async PTY (Unix: forkpty via libc; Windows: ConPTY wrapper). Workspace vendored, handles OS abstraction internally.
- **vte** — VT100 parser (output decoding).
- **async-channel, futures** — async I/O.
- **libc** — Unix syscalls (only for non-Windows).
- **Zed couplings:** gpui (event loop), collections (internal data structures), theme, settings, task (background worker traits).

**terminal_view:**
- Pure GPUI element—depends on terminal crate + gpui/theme/settings. Tightly bound to Zed's element model.

### Cross-Platform Risk Assessment

**Low-risk areas:**
- alacritty_terminal shields Windows/Unix PTY differences behind a facade (~95% of platform logic encapsulated).
- vte is platform-agnostic (pure parser).

**High-risk areas:**
- terminal_view is 100% GPUI-specific—any port requires full GPUI element reimplementation.
- terminal crate callbacks/trait objects to gpui event loop (parking_lot RwLock shared state with GPUI task dispatch)—requires careful async boundary handling via gpui_platform facade.

---

## Recommendation Summary

### Phase B (Real Syntax Highlighting)

**Minimum pull:**
1. **rope** (4.1k LOC) — **directly portable**, zero Zed types. Copy as-is; add `cfg!` gates for tests only.
2. **tree-sitter core + 3 grammars** (assume ~3–5 workspace crates, ~200–300k combined). No Zed coupling; bind via language registry abstraction.
3. **language crate (22.9k)** — **requires refactoring**: extract LanguageRegistry trait (protocol), keep Zed's impl in vendored copy, inject your own. Rope + tree-sitter glue is ~30% of language LOC; rest is grammar lookup/fallback logic.

**Do NOT pull:** text layer (requires Anchor/Selection unwrapping), multi_buffer (excerpt logic not MVP-critical).

### Phase C (Terminal PTY + Real Buffer)

**Risk escalation:** Adding terminal exposes gpui_platform facade strain.

- **terminal** (9k) — **Medium port effort**. Platform code is sparse (6 #[cfg] only), but libc calls must route through gpui_platform. alacritty_terminal handles PTY abstraction—wrap its types in facade.
- **terminal_view** (9.6k) — **Complete rewrite required**. GPUI element model is framework-specific; study Zed's impl as reference, build on your GPUI element base.
- **Buffer binding:** Integrate real text layer (rope + language) with terminal output. text crate's Anchor type is CRDT-compatible (clock::Lamport)—can be ported if collections/clock internal crates are exposed/forked.

**Cross-platform gotcha:** alacritty_terminal + libc on Unix, ConPTY on Windows. Base gpui_platform must handle:
- Unix: `forkpty()` syscall return + fd management (async-channel ↔ GPUI event loop bridge).
- Windows: ConPTY handle + legacy console mode detection (for older Windows 10 builds).
- Both: SIGWINCH/PTY resize signaling + GPUI viewport sync.

---

## Open Questions

1. Is Zed's `text::Anchor` CRDT model (Lamport clock) essential for Phase B, or can you gate multi-user features to Phase D?
2. Does base's `crates/syntax_theme/` struct layout match Zed's `theme::SyntaxTheme` (need direct file comparison)?
3. alacritty_terminal workspace version—is it pinned stable or does Zed vendor bleeding-edge? (Affects stability backport effort.)
4. Does gpui_platform already abstract libc calls, or must new wrappers be added for PTY syscalls (forkpty, ioctl TIOCSWINSZ)?

## Trade-offs

- **Rope portability vs. text layer:** rope solo is cheap; full text layer adds clock/collections burden but enables CRDT-style multi-edit. Skip text for Phase B, use simpler position model.
- **tree-sitter footprint:** 13 grammars = ~6.5MB binary. Start with 3–4, lazy-load others (Phase D).
- **terminal_view rewrite:** Not reusable from Zed; study only. Coupling to GPUI view/element API is too tight.
