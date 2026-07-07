# Phase 05 — Bump version + publish crates.io ⚠️ GATE

**⚠️ BƯỚC BẤT KHẢ NGHỊCH — outward-facing. BẮT BUỘC user go-ahead trước khi `cargo publish`.** crates.io không cho overwrite/unpublish version.

**Điều kiện tiên quyết:** Phase 04 xanh hoàn toàn (check-all + screenshot verify).

## Yêu cầu
1. **Bump version**: `base/crates/ui/Cargo.toml` `version = "0.2.8"` → `"0.2.9"`.
   - Kiểm tra closure phụ thuộc: thay đổi CHỈ trong crate `ui` → chỉ cần republish `boltz-ui`. Các crate nền (rope/language/theme/icons/gpui) KHÔNG đổi → không bump. Xác nhận `tab.rs`/`tab_bar.rs`/`pane` không thêm dep mới tới crate chưa publish.
2. **publish-crates.sh**: xem `base/script/publish-crates.sh` (hoặc tên tương đương) — nếu script publish theo thứ tự topo, chỉ cần chạy phần `boltz-ui`, hoặc `cargo publish -p boltz-ui`. Verify `cargo publish --dry-run -p boltz-ui` sạch TRƯỚC.
3. **GATE**: dừng, báo user diff + version, xin go-ahead. Chỉ publish khi user xác nhận.
4. **Publish**: `cargo publish -p boltz-ui` (từ base workspace, đã login crates.io).
5. **Verify sparse index**: chờ index cập nhật, kiểm tra `https://index.crates.io/bo/lt/boltz-ui` chứa `0.2.9` (JSON API `crates.io/api` từng bị 403 — dùng sparse index như memory).
6. Commit thay đổi base trên branch (memory: `feat/boltz-ui-split-pane-layout`) với message conventional.

## Chú ý / rủi ro
- Nếu `--dry-run` báo thiếu metadata/dep chưa publish → dừng, xử lý trước, KHÔNG publish nửa vời.
- Version đã publish là vĩnh viễn; nếu sau đó terminal lộ lỗi → phải bump 0.2.10 (không sửa 0.2.9).

## Success criteria
- `cargo publish --dry-run -p boltz-ui` sạch.
- User go-ahead nhận được.
- boltz-ui 0.2.9 hiện trên crates.io + verify qua sparse index.
