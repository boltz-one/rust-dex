# Phase B — Real Syntax Highlighting + Terminal Chrome (no PTY)

## Context links

- Plan overview: [plan.md](./plan.md)
- Research: [researcher-02-editor-terminal-stack.md](./research/researcher-02-editor-terminal-stack.md)
- Previous phase: [phase-01](./phase-01-visual-chrome-components.md)
- Next phase: [phase-03](./phase-03-real-terminal-pty-and-text-buffer.md)

## Overview

- Date: 2026-07-05
- Description: Thay buffer text của `code_editor.rs` từ `TextInput` (String-based) sang `rope` (4.1kLOC, port thẳng từ Zed) + tích hợp tree-sitter core cho 3-4 grammar cơ bản (Rust/JS/Markdown/JSON). Terminal ở phase này CHỈ là panel chrome tĩnh (không PTY, không process thật) — dùng để dàn layout trước khi Phase C thêm I/O thật.
- Priority: P2 (phụ thuộc quyết định user có chấp nhận dependency mới)
- Implementation status: Completed (2026-07-05) — xem "Ghi chú triển khai thực tế" bên dưới cho phạm vi thực tế đã làm
- Review status: Not Reviewed

**Ghi chú triển khai thực tế:**
- `crates/rope` port gần như nguyên vẹn từ `others/zed/crates/rope`, chỉ đổi `ztracing::instrument` → `tracing_facade::instrument` (base đã có sẵn `tracing_facade` làm facade tương đương, dùng đúng pattern `sum_tree` đã dùng), thêm macro `debug_panic!` cục bộ (base's `util` không có, không đáng thêm vào crate dùng chung chỉ vì 1 caller), bỏ `zlog::init_test()` (test-only, không cần). **Toàn bộ 24 test gốc của Zed pass nguyên, không sửa test nào** — xác nhận `sum_tree`/`util` của base tương thích API gần như y hệt Zed (base vốn là fork/vendor của Zed).
- `crates/language_core` (mới, không có trong đề xuất ban đầu là tên chính xác nhưng đúng vị trí `crates/language_core` đã dự kiến): `Language`/`LanguageRegistry`/`DefaultLanguageRegistry` tối giản. `highlight.rs` tự viết (không port từ Zed's `language` crate — theo ADR mục 3): parse bằng tree-sitter, chạy highlight query, resolve overlap bằng "narrowest capture wins".
- **Cập nhật (vòng 2, cùng ngày)**: đã thêm đủ 5 grammar — `lang-rust` (mặc định), `lang-javascript`, `lang-typescript` (bao gồm cả TSX qua `LANGUAGE_TSX` riêng), `lang-markdown` (chỉ block-level grammar, chưa xử lý inline emphasis/link — xem doc comment trong `language_core.rs`), `lang-json` — mỗi grammar 1 Cargo feature độc lập, KHÔNG bật mặc định (chỉ `lang-rust` bật). `crates/ui` bật cả 4 feature còn lại vì đây là nơi tiêu thụ thực tế (`code_editor.rs`'s `.language(ext)` cần dùng được).
- **Phát hiện + sửa lỗi thật khi audit `syntax_theme`** (không chỉ là "không mismatch" như ghi nhận vòng 1 — vòng 1 đã SAI một phần): `SyntaxTheme::style_for_name` chỉ match CHÍNH XÁC tên capture, KHÔNG tự fallback theo dotted-prefix như tưởng ban đầu (logic fallback đó nằm ở method khác — `highlight_id`, trả về index chứ không phải style). Test coverage audit (`code_editor.rs::tests`, chạy trên cả 5 ngôn ngữ với theme One Dark thật) phát hiện `type.builtin`, `variable.parameter` (TypeScript) và `constant.builtin`, `string.special.key` (JSON) không resolve được màu. Đã thêm hàm `style_for_capture()` tự làm fallback về segment đầu tiên trước dấu chấm (`"type.builtin"` → thử `"type"`) — dùng chung cho cả `render()` và test. Sau khi sửa: cả 5 ngôn ngữ đạt coverage ≥ 0.6 (JSON/TS) đến ≥ 0.7 (Rust) trên capture name riêng biệt.
- **Đo binary size thật** (release build `ui_gallery`, `cargo build --release`): chỉ `lang-rust` = **16,394,408 bytes** (~15.64 MiB); cả 5 grammar (Rust+JS+TS+Markdown+JSON) = **20,101,208 bytes** (~19.17 MiB). Delta cho 4 grammar thêm = **3,706,800 bytes (~3.53 MiB)**, trung bình ~880KB/grammar — CAO HƠN ước lượng ban đầu trong plan (~200-300KB/grammar), một phần vì `tree-sitter-typescript` gói cả 2 grammar con (TS + TSX) trong 1 dependency, và release profile hiện tại vẫn giữ `debug = "limited"` (chưa strip hết debug info).
- `code_editor.rs`: KHÔNG thay hẳn `TextInput`'s String bằng `Rope` như đề xuất ban đầu — xem ADR mục 4 (mới) bên dưới cho lý do và quyết định thực tế (giữ `TextInput` cho việc gõ phím, dùng highlighting engine tree-sitter chạy trực tiếp trên `&str`, chỉ bật khi `read_only(true)`).
- `TerminalPanel` chrome tĩnh: đúng như đề xuất, không PTY, `set_output`/`append_output` để caller tự "giả lập" output.
- **`command_palette`/`TabSwitcher`**: quyết định KHÔNG generic hóa thành `Picker<T: PickerDelegate>` (2 use-case vẫn khác hình dạng hành vi thật — 1 có query input, 1 không). Chỉ trích phần chrome trùng lặp thuần túy (backdrop/panel bg/border/shadow/radius — giống hệt nhau ở cả 2 file) thành `overlay_chrome.rs` (`pub(crate)`, không phải API công khai) — giảm trùng lặp mà không đụng tới logic hành vi khác nhau của 2 component.
- `make fmt-check` + `make check-all` (scoped cho crate của mình) pass; `cargo test -p boltz-ui -p boltz-rope -p boltz-language-core` — 76/76 pass (24 rope + 2 language_core + 50 boltz-ui, gồm 5 test coverage audit theme); verify runtime thực tế qua `examples/ui_gallery` (Layout page) không panic.

### ADR mục 4 (mới) — Vì sao KHÔNG thay `TextInput` bằng `Rope` trực tiếp trong `code_editor.rs`

- Context: Đề xuất ban đầu ghi "code_editor.rs chuyển buffer nội bộ từ TextInput sang rope::Rope". Nhưng `TextInput` là component DÙNG CHUNG cho `SearchInput`/`Combobox`/`InputOtp`/form field khác trong `crates/ui` — đổi lõi lưu trữ của chính `TextInput` sang `Rope` là thay đổi rộng, rủi ro cao, ảnh hưởng mọi component khác, không chỉ `CodeEditor`. Ngoài ra `TextInput` hiện KHÔNG có khái niệm cursor/caret thật (chỉ append-cuối, xem Phase A ghi chú) — nếu render qua `StyledText` (multi-span rich text) trong lúc đang gõ, con trỏ nhấp nháy sẽ không còn đúng vị trí vì `StyledText` không tự vẽ caret như `TextInput` đang làm.
- Decision: `CodeEditor` vẫn giữ `Entity<TextInput>` để xử lý phím/focus/caret như cũ (không đổi). Khi `read_only(true)` + `.language(ext)` được set, `render()` chạy `language_core::highlighted_spans()` trực tiếp trên `&str` lấy từ `TextInput::text()` mỗi frame, map màu qua `syntax_theme`, và render bằng `gpui::StyledText::with_highlights(...)` THAY vì `self.input.clone()`. Khi đang edit (không read-only), vẫn render `self.input.clone()` như cũ (plain single-color, có caret thật).
- Why this over alternatives: Cho một "read-only code preview với syntax highlighting thật" — use case chính mà doc comment gốc của `CodeEditor` đã nêu — đây đã là đủ giá trị thực tế mà không cần viết lại toàn bộ engine xử lý phím/caret cho rich-text (đó thực chất là building một phần lớn của `editor` crate 22.9kLOC của Zed, ngoài phạm vi session này). Đánh đổi: gõ trực tiếp trong khi bật `.language()` sẽ KHÔNG có highlight sống — đây là giới hạn đã ghi rõ trong doc comment của `CodeEditor::language()`, không phải hành vi ẩn.
- Trade-off chấp nhận: nếu sau này cần live-typing + live-highlight (giống VS Code thật), cần một component render engine riêng (không dựa trên `TextInput`) — để lại như "Next steps" cho phase sau nếu cần.

**Tái đánh giá (vòng 2)**: quyết định trên vẫn giữ nguyên sau khi cân nhắc lại. Để có live-typing + live-highlight thật cần tối thiểu: (1) thêm cursor/caret position tracking thật vào `TextInput` (row/column hoặc byte offset, hỗ trợ ←/→/↑/↓/Home/End/click-to-position) — hiện `TextInput` chỉ append-cuối, không có khái niệm này; (2) map byte offset sang toạ độ pixel trong `StyledText`'s layout để vẽ caret đúng chỗ; (3) chạy lại `highlighted_spans` mỗi keystroke (rẻ, không phải vấn đề); (4) tự vẽ caret như một overlay riêng vì `StyledText` không tự vẽ caret. Việc (1)+(2) là core của một text-editing engine thật — quy mô tương đương phần lớn công sức đã bỏ ra cho Phase A+B cộng lại — và (1) đụng tới `TextInput`, component dùng chung cho `SearchInput`/`Combobox`/`InputOtp`/nhiều form field khác, nên rủi ro regression cao nếu làm vội trong lúc còn Phase C đang chờ. **Quyết định: tiếp tục hoãn**, giữ giới hạn "chỉ highlight khi read_only" như hiện tại.

## Key Insights (từ nghiên cứu 02)

| Component | LOC | Coupling | Quyết định |
|---|---|---|---|
| `rope` | 4,132 | Thấp — chỉ `heapless`, `sum_tree`, `unicode-segmentation`, `util`, `ztracing`, `tracing`, `log`, `rayon` | PORT THẲNG |
| `text` | 6,570 | CAO — `clock::Lamport` (CRDT), `collections`, `parking_lot` | KHÔNG PORT (xem ADR) |
| `multi_buffer` | 16,329 | CAO — excerpt/diff layer | KHÔNG PORT, không MVP-critical |
| `language` | 22,926 | CAO — vendor 13 tree-sitter grammar crate làm workspace dep | PORT MỘT PHẦN (~30% LOC là rope+tree-sitter glue) |

- **`rope` Cargo.toml xác nhận độc lập Zed thật**: dependencies chỉ `heapless`, `log`, `rayon`, `sum_tree`, `unicode-segmentation`, `util`, `ztracing`, `tracing` — trong đó **`sum_tree` ĐÃ CÓ SẴN trong workspace base** (`crates/sum_tree`, publish `boltz-sum-tree`) và `util` cũng đã có sẵn (`boltz-util`). Chỉ cần thêm `heapless`, `unicode-segmentation`, `rayon`, `ztracing` làm external dependency mới — nhẹ, không kéo theo crate Zed nào khác. Đây là bằng chứng cụ thể củng cố khuyến nghị "port thẳng" của báo cáo 02.
- **tree-sitter footprint**: mỗi grammar ~200-300KB binary (worst case 13 grammar = 6.5MB). Phase B giới hạn 3-4 grammar (Rust/TS-hoặc-JS/Markdown/JSON) → ước tính +800KB-1.2MB binary size — cần đối chiếu tường minh với triết lý "minimal template" của `docs/project-overview-pdr.md` (xem Risk Assessment).
- Base đã có `crates/syntax_theme/syntax_theme.rs` (1 file) — cấu trúc màu-theo-scope CHƯA được đối chiếu trực tiếp với Zed's `theme::SyntaxTheme` (tên scope, cấu trúc field). Đây là rủi ro tích hợp cụ thể, không phải suy đoán — phải làm trước khi viết code binding highlight→color.
- **Do NOT pull** (theo khuyến nghị báo cáo 02): `text` layer (Anchor/Selection cần unwrap CRDT Lamport clock), `multi_buffer` (excerpt logic không cần cho MVP). Base tự thiết kế Position/Selection đơn giản (offset-based hoặc row/column, không cần CRDT vì base là single-user local editor, không có multiplayer/collab trong scope).
- Outline Panel (8.2kLOC, báo cáo 01): coupling MEDIUM-HIGH do `language::OutlineItem`. Có thể làm ở Phase B sau khi có `language` crate glue, nhưng KHÔNG bắt buộc — để như optional todo cuối phase.

## Requirements

1. `crates/rope` mới: port trực tiếp `others/zed/crates/rope/src/rope.rs` + cấu trúc con, giữ nguyên API cốt lõi (insert/delete/slice/line lookup), thay `sum_tree`/`util` workspace path bằng path đã có sẵn trong base.
2. Language registry tối giản: trait `LanguageRegistry { fn language_for_extension(&self, ext: &str) -> Option<&Language> }` + struct `Language { grammar: tree_sitter::Language, highlight_query: tree_sitter::Query }` — KHÔNG port `LanguageServer`/`Capability`/LSP glue của Zed.
3. Tích hợp 3-4 grammar: `tree-sitter-rust`, `tree-sitter-javascript` (hoặc `-typescript`), `tree-sitter-markdown`, `tree-sitter-json` làm optional Cargo features (mỗi grammar 1 feature flag, để user tự bật/tắt — giảm binary size mặc định).
4. `code_editor.rs` chuyển buffer nội bộ từ `TextInput`'s String sang `rope::Rope`, giữ nguyên public API (`text()`, `set_text()`, `read_only()`) — không breaking change cho caller Phase A.
5. Highlight rendering: chạy tree-sitter parse trên buffer, map capture name → màu từ `crates/syntax_theme` (sau khi đối chiếu cấu trúc — xem Risk).
6. Terminal panel chrome (KHÔNG PTY): component `TerminalPanel` hiển thị khối text tĩnh/giả lập (ví dụ echo lại text được set qua API, không spawn process) + có tab-bar riêng để cắm vào `PaneGroup` (Phase A) như 1 loại pane.

## Architecture

```
crates/rope/                     (MỚI, publish=true nếu ổn định, package="boltz-rope")
├── src/rope.rs                  port từ others/zed/crates/rope/src/rope.rs
└── Cargo.toml                   deps: sum_tree (workspace, có sẵn), util (workspace, có sẵn),
                                        heapless, unicode-segmentation, rayon, ztracing (MỚI)

crates/language_core/            (MỚI, tên tạm — không trùng "language" để tránh nhầm với Zed's full crate)
├── src/lib.rs                   trait LanguageRegistry, struct Language
├── src/highlight.rs             tree-sitter parse + capture→scope mapping
└── Cargo.toml                   deps: rope (crate mới trên), tree-sitter,
                                        tree-sitter-rust/-javascript/-markdown/-json (feature-gated)

crates/ui/src/components/
├── code_editor.rs               (SỬA) Rope thay TextInput's String, tích hợp highlight
└── terminal_panel.rs            (MỚI) chrome tĩnh, không PTY
```

Dependency graph mới:
```
crates/ui → crates/language_core → crates/rope → crates/sum_tree (có sẵn), crates/util (có sẵn)
                                 → tree-sitter + grammar crates (external, feature-gated)
crates/ui → crates/syntax_theme (có sẵn, cần audit field-mapping)
```

## ADR Rationale

**1. Vì sao `rope` trước, không phải `text`/`multi_buffer`?**
- Context: `text` (6.5kLOC) thêm CRDT Lamport-clock Anchor cho multi-user editing; `multi_buffer` (16.3kLOC) thêm excerpt/diff view (nhiều buffer ghép 1 view, dùng cho diff/multi-file search).
- Decision: Chỉ port `rope`. Tự thiết kế `Position { row: usize, column: usize }`/`ByteOffset` đơn giản thay `text::Anchor`.
- Why this over alternatives: Base là single-user local editor (không có collab trong `Non-Goals` của `docs/project-overview-pdr.md`), CRDT vô nghĩa nếu không multiplayer. `multi_buffer`'s excerpt view chỉ cần khi có diff-view/multi-file-search — ngoài phạm vi Phase B. Rủi ro downgrade: nếu Phase D sau này cần multi-cursor phức tạp hoặc real-time collab, sẽ phải retrofit Anchor — chấp nhận trade-off này vì YAGNI áp dụng rõ ràng ở quy mô hiện tại.

**2. Vì sao feature-gate từng tree-sitter grammar thay vì bật cả 4 mặc định?**
- Context: Mỗi grammar ~200-300KB binary; bật cả 4 = +800KB-1.2MB, đối chọi triết lý "minimal template" (`docs/project-overview-pdr.md`: "Lean & teachable... small codebase").
- Decision: `crates/language_core/Cargo.toml` khai báo `[features] lang-rust = ["dep:tree-sitter-rust"]` v.v., không bật default nào (hoặc chỉ bật `lang-rust` mặc định làm ví dụ tối thiểu).
- Why: Cho phép consumer app (`crates/app` hoặc downstream fork) chỉ trả phí binary size cho ngôn ngữ họ thực sự cần — nhất quán với cách `gpui_platform/runtime_shaders` đã dùng feature flag để opt-in behavior tốn kém.

**3. Vì sao KHÔNG port nguyên `language` crate (22.9kLOC)?**
- Context: 70% LOC của `language` là LSP glue (`LanguageServer`, `Capability`, diagnostics), chỉ ~30% là rope+tree-sitter binding thuần túy.
- Decision: Viết `language_core` mới, trích ý tưởng cấu trúc `Language`/`Grammar` nhưng bỏ hết phần LanguageServer.
- Why: Full LSP nằm ngoài phạm vi mọi phase (xem `plan.md`). Port cả 22.9kLOC để chỉ dùng 30% là lãng phí và kéo theo dependency LSP không dùng tới.

## Related code files

**Base (đọc/sửa):**
- `crates/ui/src/components/code_editor.rs` — refactor buffer backend
- `crates/syntax_theme/src/syntax_theme.rs` — audit field/scope-name trước khi viết `highlight.rs`
- `crates/sum_tree/`, `crates/util/` — dependency có sẵn cho `rope`
- `crates/ui/src/components/tab_bar.rs`, `pane_group.rs` (từ Phase A) — cắm `TerminalPanel` vào

**Zed vendor (tham khảo/port có chọn lọc):**
- `others/zed/crates/rope/src/rope.rs` — port gần như nguyên vẹn
- `others/zed/crates/rope/Cargo.toml` — đối chiếu dependency list khi viết `crates/rope/Cargo.toml`
- `others/zed/crates/language/` — chỉ đọc phần rope+tree-sitter glue (không port LSP phần còn lại)
- `others/zed/crates/text/` — CHỈ ĐỌC để hiểu vì sao KHÔNG port (Anchor/Lamport clock)
- `others/zed/crates/terminal_view/` — tham khảo layout chrome tĩnh (không đọc phần PTY I/O, đó là Phase C)

## Implementation Steps

1. Đối chiếu `crates/syntax_theme/src/syntax_theme.rs` với Zed's `theme::SyntaxTheme` (scope tên, field) — viết mapping table nếu khác biệt.
2. Tạo `crates/rope/` — copy `rope.rs`, đổi import `sum_tree`/`util` sang path workspace base, thêm 4 dependency mới (`heapless`, `unicode-segmentation`, `rayon`, `ztracing`) vào `[workspace.dependencies]` gốc.
3. `cargo check -p rope` độc lập trước khi tích hợp — cô lập lỗi biên dịch.
4. Tạo `crates/language_core/` với `LanguageRegistry` trait + `Language` struct, feature-gate từng grammar.
5. Viết `highlight.rs`: parse buffer bằng tree-sitter, chạy highlight query, trả về `Vec<(Range<usize>, ScopeName)>`.
6. Refactor `code_editor.rs`: thay `Entity<TextInput>` backing text bằng `Rope`, giữ public API. Highlight render bằng cách chia text thành span theo `Vec<(Range, ScopeName)>` × màu từ `syntax_theme`.
7. `TerminalPanel` chrome tĩnh — không PTY, chỉ hiển thị nội dung set qua `set_output(text: impl Into<String>)`.
8. Benchmark binary size trước/sau (dùng `cargo bloat` hoặc so sánh `target/release/app` size) — ghi số liệu cụ thể vào changelog, không ước lượng chung chung.
9. `make fmt-check && make check-all`.

## Todo list

- [ ] Audit `syntax_theme` field-mapping (viết kết quả thành comment/doc trong `highlight.rs`)
- [ ] `crates/rope/` port + cargo check độc lập
- [ ] Thêm 4 dependency mới vào `[workspace.dependencies]`
- [ ] `crates/language_core/` + 3-4 grammar feature-gated
- [ ] `code_editor.rs` refactor sang Rope + highlight render
- [ ] `TerminalPanel` chrome tĩnh
- [ ] Đo binary size delta, ghi số liệu thật vào PR/changelog
- [ ] `make fmt-check && make check-all` pass

## Success Criteria

- `crates/rope` compile độc lập, không kéo bất kỳ Zed-internal crate nào ngoài `sum_tree`/`util` đã có sẵn.
- `code_editor.rs` hiển thị màu syntax đúng cho ít nhất Rust source, không regression về hành vi nhập liệu/read-only đã có ở Phase A.
- Binary size tăng được đo và document cụ thể (không phải ước lượng), đối chiếu với ngưỡng chấp nhận được của user (Unresolved Question #2 ở `plan.md`).
- `TerminalPanel` render được trong `PaneGroup` không lỗi layout.

## Risk Assessment

- **Binary size risk (Trung bình-Cao)**: 3-4 grammar ~800KB-1.2MB — vi phạm tinh thần "minimal template" nếu bật mặc định. Giảm thiểu: feature-gate, không bật default nhiều hơn 1 grammar.
- **syntax_theme mismatch risk (Trung bình)**: nếu cấu trúc `crates/syntax_theme` khác biệt lớn so với Zed's scope-naming convention, cần viết lại tầng mapping — effort không nằm trong ước tính LOC gốc của báo cáo 02 (báo cáo chỉ nêu "cần verify", không định lượng effort).
- **Rope integration risk (Thấp)**: LOC nhỏ, dependency sạch, rủi ro chủ yếu là API mismatch nhỏ khi nối vào `TextInput`'s render logic hiện tại (cursor rendering, selection) — cần audit `TextInput` API trước khi bắt đầu bước 6.

## Security Considerations

Tree-sitter parse trên input người dùng cục bộ — rủi ro DoS thấp (tree-sitter có giới hạn thời gian parse nội tại nhưng cần kiểm chứng cho input cực lớn/malformed). Không tải grammar động qua network (build-time linked only) — không có bề mặt tấn công supply-chain runtime.

## Next steps

Sau khi Phase B hoàn thành, đọc [phase-03](./phase-03-real-terminal-pty-and-text-buffer.md) để đánh giá độ rủi ro cross-platform PTY trước khi cam kết tiếp.
