# Goal: Zed Tab Styling Parity

## Mission
Map chính xác 1:1 styling tab của Zed sang tab của `terminal/` (render qua boltz-ui `base/crates/ui`), validate trong base, publish boltz-ui 0.2.9 lên crates.io, `terminal/` consume + screenshot-verify khớp Zed.

## Context & Key Files
- Full plan: `base/plans/20260706-1756-zed-tab-styling-parity/plan.md`
- Phases 01→06: `base/plans/20260706-1756-zed-tab-styling-parity/phase-0*.md`
- Mapping table (token/size/border): `.../research/zed-boltz-tab-mapping.md`
- Sửa: `base/crates/ui/src/components/{tab.rs, tab_bar.rs, pane/render.rs}`, `base/crates/ui/Cargo.toml`, `terminal/Cargo.toml`
- Tham chiếu Zed: `others/zed/crates/ui/src/components/{tab.rs, tab_bar.rs}`, `workspace/src/pane.rs`

## Requirements
**Must do (bám phase files để có code cụ thể):**
- P01 `tab.rs`: port `TabPosition{First,Middle(Ordering),Last}` + `TabCloseSide`; rewrite nhánh `Underline` → active bg=`tab_active_background`+bỏ border đáy(`pb_px`), inactive bg=`tab_inactive_background`+`border_b_1`, màu viền=`border`; padding=`Base04`, gap=`Base04`, slot 12/14px.
- P02 `tab_bar.rs`: bg=`tab_bar_background`; thêm overlay kẻ đáy `div().absolute().top_0().left_0().size_full().border_b_1()` làm child ĐẦU (dưới tabs); start/end slot thêm `border_b_1`+`border_r_1`/`border_l_1`.
- P03 `pane/render.rs`: tính `TabPosition` mỗi tab (`ix.cmp(&active_idx)`); ✕ = `IconSize::Small`+`Color::Muted`+`IconButtonShape::Square`+`ButtonSize::None`; `visible_on_hover` cho MỌI tab (bỏ nhánh `if !selected`).
- P04: `make check-all`+`fmt-check` xanh; test `pane_group_harness.rs`+`pane_group_probe_e2e.rs` xanh; screenshot đối chiếu Zed.
- P05 (GATE): bump `boltz-ui` 0.2.8→0.2.9, `cargo publish --dry-run -p boltz-ui` sạch, **xin user go-ahead** rồi publish, verify sparse index.
- P06: `terminal/Cargo.toml` dep→0.2.9, `cargo update -p boltz-ui --precise 0.2.9`, build+screenshot-verify.

**Must not:**
- KHÔNG đụng nhánh `TabBarStyle::Pills`.
- KHÔNG thêm icon terminal vào tab; KHÔNG đổi `LabelSize::Small`; KHÔNG thêm `min_w` (parity 1:1) — chỉ thêm lại nếu screenshot lộ clip thật.
- KHÔNG thêm `.overflow_x_hidden()` (dù Zed có — làm tab non-hit-testable, đã kiểm chứng).
- KHÔNG path-depend base cục bộ ở terminal; KHÔNG publish khi chưa có go-ahead (bất khả nghịch).

## Success Criteria
- `cd base && make check-all && make fmt-check` → exit 0.
- Test file `pane_group_harness.rs` + `pane_group_probe_e2e.rs` PASS (chạy theo file, KHÔNG aggregate `-p ui_gallery`; `visual_harness.rs` SIGABRT có sẵn — bỏ qua).
- Screenshot: tab active=bg sáng+nối content (KHÔNG accent xanh), inactive=muted+kẻ đáy, ✕ ẩn khi không hover.
- `boltz-ui 0.2.9` hiện trên sparse index `index.crates.io/bo/lt/boltz-ui`.
- `cd terminal && cargo build` → exit 0 với dep 0.2.9; app chạy, tab khớp Zed.

## Out of Scope
- Drag cross-pane; render PTY thật; search/filter thật; narrow rail.
- Bất kỳ crate nền nào ngoài `boltz-ui` (rope/language/theme/icons/gpui không bump).

## Verification
```bash
cd base && make check-all && make fmt-check
cd base && cargo test -p ui_gallery --test pane_group_harness && cargo test -p ui_gallery --test pane_group_probe_e2e
curl -s https://index.crates.io/bo/lt/boltz-ui | grep -q '"vers":"0.2.9"' && echo "published ok"
cd terminal && cargo update -p boltz-ui --precise 0.2.9 && cargo build
```
