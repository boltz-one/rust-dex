# Tailwind Plus Application UI Complete Catalog

**Purpose:** Full component inventory for GPUI UI kit gap analysis  
**Date:** 2026-07-03  
**Token Source:** Reuse from `researcher-01-tailwind-spec.md` (px() + hsla())

---

## Application UI Component Matrix

| **Section** | **Category** | **Core Variants/Anatomy** | **Token Profile** | **GPUI Fit** |
|---|---|---|---|---|
| **Application Shells** | Stacked Layout | Header + sidebar + main content | bg white, border gray-200, shadow-sm | ✓ Desktop native |
| | Sidebar + Content | Fixed sidebar (256px), collapsible | bg white/gray-900, border-r gray-200 | ✓ Core |
| | Multi-column Grid | 2-3 col flex layouts | gap-6, px-6, py-4 | ✓ Core |
| **Headings** | Page Heading | Title (3xl/30px) + subtitle + actions | text-gray-900, mb-6, flex justify-between | ✓ Core |
| | Card Heading | Title (lg/18px) + badge/status | text-gray-700, mb-3, flex items-center | ✓ Core |
| | Section Heading | Title (2xl/24px) grouped content | text-gray-800, mb-4 | ✓ Core |
| **Data Display** | Description Lists | Key-value stacked/horizontal | border-t gray-200, py-4 px-4 | ✓ Desktop |
| | Stats Cards | Metric + label, 2-4 col grid | bg white, border gray-200, rounded-lg | ✓ Core |
| | Calendars | Month grid + day cells, interactive | border gray-300, text-sm, hover:bg-gray-100 | ⚠ Complex JS, low priority |
| **Lists** | Stacked List | Item row, icon + text + action | border-b gray-200, py-4 px-4, hover:bg-gray-50 | ✓ Core |
| | Tables | Header + striped rows, sort/pagination | bg-gray-50, border-b gray-200, text-sm | ✓ Core |
| | Grid List | Card-based (2-4 col), images + text | gap-6, rounded-lg, shadow-sm | ✓ Core |
| | Feeds | Timeline/activity log, avatars | border-l gray-200, pl-4, mb-4 | ✓ Desktop |
| **Forms** | Form Layouts | Vertical/horizontal field groups | gap-6, mb-6, block | ✓ Core |
| | Input Groups | Input + prefix/suffix button | border gray-300, rounded-md, px-3 py-2 | ✓ Core |
| | Select Menus | Dropdown (custom), chevron icon | border gray-300, rounded-md, focus:ring-2 | ✓ Core |
| | Sign-in/Registration | Full-page forms (email + password) | max-w-md, centered, gap-6 | ⚠ Web layout, consider desktop variant |
| | Textareas | Multi-line input, resize | border gray-300, rounded-md, min-h-6rem | ✓ Core |
| | Radio Groups | Inline/stacked options | flex gap-4, border gray-300, checked:bg-blue-600 | ✓ Core |
| | Checkboxes | Single/batch select, indeterminate | size-4, border gray-300, checked:bg-blue-600 | ✓ Core |
| | Toggles/Switches | Binary state (w-44 h-24) | bg-gray-200/blue-600, smooth transition | ✓ Core |
| | Action Panels | Fieldset + buttons (save/cancel) | border-t gray-200, pt-6, flex justify-end gap-3 | ✓ Core |
| | Comboboxes | Searchable select, dropdown | border gray-300, rounded-md, bg-white | ✓ Core |
| **Feedback** | Alerts | 4 types (info/success/warning/error) | border-l-4, padding px-4 py-3, icons | ✓ Core |
| | Alert Variants | Solid/outline/soft styles | bg-{color}-50/100, text-{color}-800/700 | ✓ Core |
| | Empty States | Illustration + heading + action | text-center, py-12, icon 48px | ✓ Desktop |
| **Navigation** | Navbars | Horizontal, sticky top, logo + menu | bg white, border-b gray-200, px-6 py-4 | ✓ Core |
| | Pagination | Previous/next + numbered | flex gap-1, button variants, disabled state | ✓ Core |
| | Vertical Navigation | Sidebar links, active state | px-4 py-2, rounded-md, hover:bg-gray-100 | ✓ Core |
| | Sidebar Navigation | Dark/light, collapsed icons | fixed w-64, border-r, link active bg-gray-800 | ✓ Core |
| | Breadcrumbs | Path navigation, separators | flex gap-2, text-sm, text-gray-500 | ✓ Core |
| | Tabs | Underline + pills styles | flex gap-8, border-b gray-200, py-4 | ✓ Core |
| | Progress Bars | Linear progress indicator | bg-gray-200, relative h-2, rounded-full | ✓ Core |
| | Command Palettes | Searchable command menu (modal-based) | bg white, input search, item list, keyboard | ⚠ Complex interaction, medium priority |
| **Overlays** | Modal Dialogs | Center overlay, max-w 448-672px | bg white, rounded-lg, shadow-xl, ring z-50 | ✓ Core |
| | Drawers/Slide-overs | Side panel, slide animation | fixed right-0, bg white, shadow-xl, w-96 | ✓ Core |
| | Notifications/Toasts | Fixed bottom-right, auto-dismiss 5s | bg white, border gray-200, shadow-lg, p-4 | ✓ Core |
| **Elements** | Avatars | 6 sizes (24-64px), initials/image/icon | rounded-full, bg-{color}-600, text white, border | ✓ Core |
| | Avatar Groups | Overlapped (negative margin) | margin-ml-2 (offset), grouped flex | ✓ Core |
| | Badges | Solid/soft/outline/dot variants | px-2 py-1, text-xs font-medium, rounded-full | ✓ Core |
| | Dropdowns | Button trigger + menu items | bg white, border gray-200, rounded-md, shadow-lg | ✓ Core |
| | Buttons | 7 variants (primary/secondary/soft/white/outline/ghost/danger) | 5 sizes, hover + active + disabled + loading | ✓ Core |
| | Button Groups | Segmented controls, connected layout | flex, border gray-300, focus:ring-2 | ✓ Core |
| **Layout** | Containers | Fixed/responsive max-width centering | max-w-7xl, mx-auto, px-4 sm:px-6 | ✓ Fixed for desktop |
| | Cards | Base container (bg white, border, shadow) | border gray-200, rounded-lg, shadow-sm, p-6 | ✓ Core |
| | List Containers | Wrapper for grouped items | border gray-200, rounded-lg | ✓ Core |
| | Media Objects | Image + text side-by-side flex | gap-4, flex items-start | ✓ Core |
| | Dividers | Horizontal/vertical separation | border-t/l gray-200, my-6/mx-6 | ✓ Core |

---

## Not Suitable for Desktop GPUI (Web-centric)

| Category | Reason | Alternative |
|---|---|---|
| Sign-in/Registration (full-page layout) | Assumes full viewport + responsive breakpoints; desktop typically has fixed size. | Adapt to modal/centered card variant. |
| Calendars | Heavy JS interaction, date picker; rarely needed in initial app UI kit. | Phase 2+ after core components. |
| Command Palettes | Complex keyboard nav + filtering; more common in code editors/IDEs. | Phase 2+ if app needs command-driven UX. |
| Responsive containers (sm:/md:/lg: breakpoints) | Native desktop doesn't need adaptive breakpoints; use fixed pixel sizes. | GPUI px() only; skip Tailwind breakpoints. |

---

## Sources

1. **Tailwind Plus / TailwindUI Docs**  
   - https://tailwindcss.com/plus (redirect from tailwindui.com)  
   - [Browse all Application UI Components](https://tailwindcss.com/plus/ui-blocks/application-ui)

2. **Design Tokens Reference**  
   - https://tailwindcss.com/docs/colors  
   - https://tailwindcss.com/docs/spacing  
   - https://tailwindcss.com/docs/border-radius  
   - Reused from: `researcher-01-tailwind-spec.md`

---

## Open Questions

- **Exact component count:** Tailwind Plus claims 500+ components; full breakdown not yet enumerated per section  
- **Component parity:** Which Tailwind UI components have direct equivalents vs. partial/custom builds needed for GPUI?  
- **Icon library:** Tailwind UI assumes Heroicons; GPUI embed strategy TBD (SVG bundle, system icons, custom set)  
- **Responsive variants:** Desktop fixed-size approach confirmed; verify if any component requires adaptive sizing  
- **Color space (v4 OKLCH):** Use hex (current) or await OKLCH precision for GPUI rendering  

---

**Next Step:** Gap-analysis matrix mapping each GPUI requirement to Tailwind component(s) for priority roadmap.
