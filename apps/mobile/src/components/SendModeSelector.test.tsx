import { render, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it, vi } from 'vitest';

import type { Device } from '@dropvoice/core';

import { SendModeSelector } from './SendModeSelector';

const baseDevice: Omit<Device, 'id' | 'name'> = {
  autoConnect: true,
};

function makeDevice(id: string, name: string): Device {
  return { ...baseDevice, id, name };
}

describe('SendModeSelector', () => {
  it('renders nothing when there is only one device (spec 10 §2.4)', () => {
    const { container } = render(
      <SendModeSelector
        mode="active"
        devices={[makeDevice('a', 'PC-1')]}
        selectedDevices={[]}
        onModeChange={() => {}}
        onSelectionChange={() => {}}
      />
    );
    expect(container).toBeEmptyDOMElement();
  });

  it('exposes the three send-mode radio buttons (active / all / selected)', () => {
    const devices = [makeDevice('a', 'PC-1'), makeDevice('b', 'PC-2')];
    render(
      <SendModeSelector
        mode="active"
        devices={devices}
        selectedDevices={[]}
        onModeChange={() => {}}
        onSelectionChange={() => {}}
      />
    );
    const radios = screen.getAllByRole('radio');
    expect(radios).toHaveLength(3);
    expect(radios[0]).toHaveAttribute('aria-checked', 'true');
    expect(radios[1]).toHaveAttribute('aria-checked', 'false');
    expect(radios[2]).toHaveAttribute('aria-checked', 'false');
  });

  it('drives onModeChange when a mode radio is clicked (controlled)', async () => {
    const user = userEvent.setup();
    const onModeChange = vi.fn();
    const devices = [makeDevice('a', 'PC-1'), makeDevice('b', 'PC-2')];
    render(
      <SendModeSelector
        mode="active"
        devices={devices}
        selectedDevices={[]}
        onModeChange={onModeChange}
        onSelectionChange={() => {}}
      />
    );
    // Three radios: active (checked), all, selected. Click on the second one.
    const radios = screen.getAllByRole('radio');
    await user.click(radios[1]);
    expect(onModeChange).toHaveBeenCalledWith('all');
  });

  it('does not show the per-device checklist until mode=selected', () => {
    const devices = [makeDevice('a', 'PC-1'), makeDevice('b', 'PC-2')];
    const { rerender } = render(
      <SendModeSelector
        mode="active"
        devices={devices}
        selectedDevices={[]}
        onModeChange={() => {}}
        onSelectionChange={() => {}}
      />
    );
    expect(screen.queryByRole('checkbox')).not.toBeInTheDocument();

    rerender(
      <SendModeSelector
        mode="selected"
        devices={devices}
        selectedDevices={[]}
        onModeChange={() => {}}
        onSelectionChange={() => {}}
      />
    );
    expect(screen.getAllByRole('checkbox')).toHaveLength(2);
  });

  it('drives onSelectionChange when a device checkbox is toggled', async () => {
    const user = userEvent.setup();
    const onSelectionChange = vi.fn();
    const devices = [makeDevice('a', 'PC-1'), makeDevice('b', 'PC-2')];
    render(
      <SendModeSelector
        mode="selected"
        devices={devices}
        selectedDevices={[]}
        onModeChange={() => {}}
        onSelectionChange={onSelectionChange}
      />
    );
    await user.click(screen.getAllByRole('checkbox')[1]);
    expect(onSelectionChange).toHaveBeenCalledWith(['b']);
  });

  it('removes a device from the selection when its checkbox is toggled off', async () => {
    const user = userEvent.setup();
    const onSelectionChange = vi.fn();
    const devices = [makeDevice('a', 'PC-1'), makeDevice('b', 'PC-2')];
    render(
      <SendModeSelector
        mode="selected"
        devices={devices}
        selectedDevices={['a', 'b']}
        onModeChange={() => {}}
        onSelectionChange={onSelectionChange}
      />
    );
    await user.click(screen.getAllByRole('checkbox')[0]);
    expect(onSelectionChange).toHaveBeenCalledWith(['b']);
  });
});
