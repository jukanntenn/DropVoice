import { render, screen } from '@testing-library/react';
import { describe, expect, it } from 'vitest';

import { Progress } from './Progress';

describe('Progress', () => {
  it('renders a progressbar with aria-valuenow matching the value', () => {
    render(<Progress value={42} aria-label="Uploading" />);
    const progressbar = screen.getByRole('progressbar', { name: 'Uploading' });
    expect(progressbar).toBeInTheDocument();
    expect(progressbar).toHaveAttribute('aria-valuenow', '42');
    expect(progressbar).toHaveAttribute('aria-valuemin', '0');
    expect(progressbar).toHaveAttribute('aria-valuemax', '100');
  });

  it('respects a custom max value', () => {
    render(<Progress value={5} max={10} aria-label="Loading" />);
    const progressbar = screen.getByRole('progressbar');
    expect(progressbar).toHaveAttribute('aria-valuemax', '10');
    expect(progressbar).toHaveAttribute('aria-valuenow', '5');
  });

  it('exposes indeterminate state when value is null', () => {
    render(<Progress value={null} aria-label="Loading" />);
    const progressbar = screen.getByRole('progressbar');
    expect(progressbar).not.toHaveAttribute('aria-valuenow');
  });
});
