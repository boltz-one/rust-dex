# Dependency Graph & Publish Order for 14 Remaining Crates

## 1. Verified Dependency Graph

Extracted from `crates/*/Cargo.toml` [dependencies] sections:

| Crate | Internal Dependencies |
|-------|----------------------|
| font_kit | (none detected) |
| syntax_theme | (none detected) |
| icons | (none detected) |
| menu | (none detected) |
| ui_macros | (none detected) |
| gpui_wgpu | (none detected) |
| gpui_windows | (none detected) |
| theme | syntax_theme |
| gpui_linux | gpui_wgpu |
| gpui_macos | font_kit |
| component | theme |
| gpui_platform | gpui_macos, gpui_linux, gpui_windows (target-cfg gated) |
| ui | component, icons, menu, theme, ui_macros |
| app | gpui_platform, theme, ui |

**Note:** cargo tree showed empty deps (likely due to `.workspace = true` refs not expanding). Extracted from Cargo.toml [dependencies] section instead. ✓

## 2. Topological Publish Order (Deps-First)

```
1. font_kit         (0 internal deps)
2. syntax_theme     (0 internal deps)
3. icons            (0 internal deps)
4. menu             (0 internal deps)
5. ui_macros        (0 internal deps)
6. gpui_wgpu        (0 internal deps)
7. gpui_windows     (0 internal deps)
8. theme            → syntax_theme
9. gpui_linux       → gpui_wgpu
10. gpui_macos      → font_kit
11. component       → theme
12. gpui_platform   → gpui_macos, gpui_linux, gpui_windows
13. ui              → component, icons, menu, theme, ui_macros
14. app             → gpui_platform, theme, ui
```

**Matches assumed list from task context exactly.** ✓

## 3. Line Numbers for Edits

### Crate Cargo.toml `name =` fields (crates/*/Cargo.toml)

| Crate | Line | Root Workspace.dependencies Entry | Line |
|-------|------|----------------------------------|------|
| font_kit | 2 | `font_kit` (line 47) | 47 |
| theme | 2 | `theme` (line 65) | 65 |
| component | 2 | `component` (line 45) | 45 |
| icons | 2 | `icons` (line 58) | 58 |
| menu | 2 | `menu` (line 60) | 60 |
| ui | 2 | `ui` (line 66) | 66 |
| app | 4 | `app` (line 178) | 178 |
| syntax_theme | 2 | `syntax_theme` (line 64) | 64 |
| ui_macros | 2 | `ui_macros` (line 67) | 67 |
| gpui_platform | 2 | `gpui_platform` (line 53) | 53 |
| gpui_linux | 2 | `gpui_linux` (line 50) | 50 |
| gpui_macos | 2 | `gpui_macos` (line 51) | 51 |
| gpui_wgpu | 2 | `gpui_wgpu` (line 54) | 54 |
| gpui_windows | 2 | `gpui_windows` (line 55) | 55 |

*All `name =` fields in crate Cargo.toml are on line 2 except `app` (line 4).*

## 4. crates.io Availability (Re-check)

All 14 proposed names returned HTTP 404 (available):

```
✓ boltz-font-kit
✓ boltz-theme
✓ boltz-component
✓ boltz-icons
✓ boltz-menu
✓ boltz-ui
✓ boltz-app
✓ boltz-syntax-theme
✓ boltz-ui-macros
✓ boltz-gpui-platform
✓ boltz-gpui-linux
✓ boltz-gpui-macos
✓ boltz-gpui-wgpu
✓ boltz-gpui-windows
```

Query method: `https://index.crates.io/<sharding>/<name>` with curl HTTP status check. No names taken since prior session. ✓

## 5. Surprises/Risks

**None detected.** Dependency graph exactly matches task context assumptions:
- ✓ theme → syntax_theme
- ✓ component → theme
- ✓ ui → {component, icons, menu, theme, ui_macros}
- ✓ app → {ui, theme, gpui_platform}
- ✓ gpui_platform → {gpui_macos, gpui_linux, gpui_windows} (target-cfg)
- ✓ gpui_linux → gpui_wgpu
- ✓ gpui_macos → font_kit

Leaf crates (font_kit, syntax_theme, icons, menu, ui_macros, gpui_wgpu, gpui_windows) confirmed zero internal deps—safe to publish first.

## Open Questions

None. All dependencies verified, line numbers confirmed, crates.io names validated available, publish order confirmed topologically correct.
