import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { Select } from './Select';

const OPTIONS = [
  { value: 'a', label: 'Alpha' },
  { value: 'b', label: 'Beta' },
  { value: 'c', label: 'Gamma' },
];

describe('Select', () => {
  it('renders a combobox trigger with the selected value', () => {
    render(<Select value="a" options={OPTIONS} onValueChange={() => {}} aria-label="Letter" />);
    expect(screen.getByRole('combobox', { name: 'Letter' })).toHaveTextContent('Alpha');
  });

  it('opens the listbox and exposes options on click', async () => {
    const user = userEvent.setup();
    render(<Select value="a" options={OPTIONS} onValueChange={() => {}} aria-label="Letter" />);
    await user.click(screen.getByRole('combobox'));
    expect(await screen.findByRole('listbox')).toBeInTheDocument();
    expect(screen.getByRole('option', { name: 'Beta' })).toBeInTheDocument();
  });

  it('calls onValueChange when an option is selected', async () => {
    const user = userEvent.setup();
    const onChange = vi.fn();
    render(<Select value="a" options={OPTIONS} onValueChange={onChange} aria-label="Letter" />);
    await user.click(screen.getByRole('combobox'));
    await user.click(await screen.findByRole('option', { name: 'Gamma' }));
    expect(onChange).toHaveBeenCalledWith('c');
  });
});
