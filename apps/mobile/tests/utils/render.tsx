/**
 * Custom `render` wrapper (spec 06 §2.4).
 *
 * Wraps `@testing-library/react::render` with any providers the mobile
 * component tree expects (jotai Provider, etc.). Use this instead of the raw
 * `render` so tests don't have to repeat provider boilerplate.
 */
import { render } from '@testing-library/react';
import type { ReactElement, ReactNode } from 'react';

import { Provider as JotaiProvider } from 'jotai';

function Wrapper({ children }: { children: ReactNode }): ReactElement {
  return <JotaiProvider>{children}</JotaiProvider>;
}

export function renderWithProviders(ui: ReactElement) {
  return render(ui, { wrapper: Wrapper });
}

export { render };
