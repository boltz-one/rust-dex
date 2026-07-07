# Phase 06 — terminal/ consume 0.2.9 + verify

**File:** `terminal/Cargo.toml` (dòng 18: `ui = { package = "boltz-ui", version = "0.2.8" }`)
**Điều kiện tiên quyết:** Phase 05 xong (0.2.9 verified trên crates.io sparse index).

## Yêu cầu
1. Bump dep: `version = "0.2.8"` → `"0.2.9"` trong `terminal/Cargo.toml`.
2. `cargo update -p boltz-ui --precise 0.2.9` (trong `terminal/`) để refresh `Cargo.lock`; verify lock trỏ 0.2.9 (không kẹt cache cũ).
3. `cargo build` trong `terminal/` sạch (không warning mới). Nếu API `Tab`/`TabBar` đổi chữ ký ảnh hưởng call-site terminal (khó vì Pane render nằm trong ui) → sửa call-site.
4. **Screenshot verify** (workflow macOS trong memory): `cargo run` terminal → split 2-3 pane/tab bằng `osascript ... keystroke "d" using {command down}` (System Events, đã có Accessibility permission) → lấy window ID qua Quartz theo PID binary → `screencapture -x -o -l<WID>`.
5. Đối chiếu tab terminal với Zed thật: active bg sáng + nối content (không accent xanh), inactive muted + kẻ đáy, ✕ hover-only, padding 4px, tab co theo nội dung.

## Chú ý / rủi ro
- Nếu terminal lộ thiếu (vd cần API mới của Tab) → không path-depend; cycle bump 0.2.10 (memory rule).
- `pgrep -x terminal` lấy PID binary (KHÔNG phải zsh wrapper) cho Quartz.

## Success criteria
- terminal build + chạy với boltz-ui 0.2.9.
- Screenshot xác nhận tab terminal khớp Zed 1:1 (trong giới hạn: giữ LabelSize::Small, không icon).
- Zero regression: split/close/reorder/nút +/close-pane hoạt động.
- Cập nhật memory `terminal-app-decisions.md`: ghi mốc "tab styling → Zed parity, boltz-ui 0.2.9".
