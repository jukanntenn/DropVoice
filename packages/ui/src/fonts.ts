/**
 * Inter webfont registration. Side-effect imports — Vite bundles the @font-face CSS
 * emitted by @fontsource/inter so the "Inter" family named in effects.css resolves.
 * Re-exported via src/index.ts, so any import from @dropvoice/ui loads the font.
 */
import '@fontsource/inter/400.css';
import '@fontsource/inter/500.css';
import '@fontsource/inter/600.css';
import '@fontsource/inter/700.css';
