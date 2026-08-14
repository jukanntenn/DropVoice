import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import { Dialog } from './Dialog';

describe('Dialog', () => {
  it('does not render the dialog title when closed', () => {
    render(<Dialog open={false} onClose={() => {}} title="My Title" />);
    expect(screen.queryByText('My Title')).not.toBeInTheDocument();
  });

  it('renders title and description when open', () => {
    render(
      <Dialog open onClose={() => {}} title="My Title" description="My description">
        Body content
      </Dialog>
    );
    expect(screen.getByText('My Title')).toBeInTheDocument();
    expect(screen.getByText('My description')).toBeInTheDocument();
    expect(screen.getByText('Body content')).toBeInTheDocument();
  });

  it('renders footer content when provided', () => {
    render(
      <Dialog open onClose={() => {}} title="My Title" footer={<button type="button">OK</button>}>
        Body
      </Dialog>
    );
    expect(screen.getByRole('button', { name: 'OK' })).toBeInTheDocument();
  });

  it('calls onClose when the close (X) button is clicked', async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    render(<Dialog open onClose={onClose} title="My Title" />);
    await user.click(screen.getByRole('button', { name: 'Close' }));
    expect(onClose).toHaveBeenCalledTimes(1);
  });

  it('calls onClose when the Escape key is pressed', async () => {
    const user = userEvent.setup();
    const onClose = vi.fn();
    render(<Dialog open onClose={onClose} title="My Title" />);
    await user.keyboard('{Escape}');
    expect(onClose).toHaveBeenCalledTimes(1);
  });
});
