# Phase 04 — Validate trong base (bắt buộc trước publish)

**Ràng buộc (memory):** cấm path-depend base để test terminal → phải chứng minh tab đúng NGAY TRONG base qua `ui_gallery` + screenshot TRƯỚC khi publish.

## Yêu cầu
1. **Build + gate**:
   - `make check-all` + `make fmt-check` (theo `base/Makefile`) phải xanh.
   - `cargo build -p boltz-ui` + `cargo build -p ui_gallery` sạch, không warning mới.
2. **Test hiện có không vỡ**:
   - `base/examples/ui_gallery/tests/pane_group_harness.rs` (8 test state) + `pane_group_probe_e2e.rs` (3 test gpui-probe) phải vẫn xanh.
   - ⚠️ KHÔNG chạy `visual_harness.rs` (pre-existing SIGABRT do TerminalView PTY thread — không phải do ta; xem memory). Chạy test theo FILE, không aggregate `cargo test -p ui_gallery`.
   - Nếu probe test khẳng định vị trí ✕ dựa trên "active luôn hiện ✕" → cập nhật assertion cho hành vi hover-only mới.
3. **Screenshot verify** (macOS, workflow trong memory):
   - `cargo run -p ui_gallery` (hoặc app gallery) → split ≥2-3 tab.
   - Lấy window ID qua Quartz theo PID binary → `screencapture -x -o -l<WID>`.
   - Đối chiếu mắt-thường với Zed thật (chạy zed hoặc ảnh tham chiếu): active=bg sáng + nối content (không accent xanh), inactive=muted + kẻ đáy, ✕ ẩn khi không hover.
4. **So sánh chéo** giá trị render vs `research/zed-boltz-tab-mapping.md` (32px height, padding 4px, ✕ 14px...).

## Chú ý
- gpui-probe chỉ probe element intrinsic (nút/label), KHÔNG probe `size_full`/flex container (memory caveat).
- Nếu screenshot lộ lệch (vd kẻ đáy chặn click, tab quá hẹp clip title với LabelSize::Small) → quay lại Phase 01-03 sửa; cycle trong base, CHƯA publish.

## Success criteria
- `make check-all` + `fmt-check` xanh.
- Test pane_group (harness + probe) xanh.
- Screenshot chứng minh tab khớp Zed (active bg-based, không accent, ✕ hover-only).
- Zero regression click/drag/reorder.
