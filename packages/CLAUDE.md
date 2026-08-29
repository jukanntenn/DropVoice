# packages — standing orders

Shared packages consumed by the apps: `core` (pure TS), `i18n` (4 locales), `ui` (design system). Version inheritance is centralized: members use `workspace = true` (Cargo) / `workspace:*` (pnpm); a shared dependency is added to the root manifest first — never pinned inline in a member (the axum 0.7/0.8 split happened once).

## Orders

- `core` holds state machines as **pure reducers** (no xstate, no side effects), jotai atoms, and shared types. Anything with effects belongs to the app consuming it.
- `i18n`: 4 locales (`en`, `zh`, `zh-TW`, `ja`) × 5 namespaces (`common`, `devices`, `errors`, `landing`, `settings`) = 20 JSON files. Detection order: querystring `?lang=` → localStorage `dropvoice-lang` → navigator. A key touched in one locale file is touched in all four, in the same change.
- `ui`: `src/tokens/theme.css` is **generated** from root `DESIGN.md` via `pnpm design:sync` and drift-gated (`design-sync`) — never hand-edit; change tokens in `DESIGN.md` and regenerate.
- Every package carries its own `prek.toml`; tests are co-located as `*.test.ts(x)` and run through prek's pre-push stage.
