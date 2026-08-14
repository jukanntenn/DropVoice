/**
 * Select primitive built on @base-ui/react (spec 07 §2.2).
 *
 * Wraps Base UI's Select with cva-driven styling. Uses Floating UI for popup
 * positioning. Mirrors base-ui's controlled API: `value` + `onValueChange`.
 * The `options` prop accepts a simple `{value,label}[]` list for ergonomics.
 */
import { Select as BaseSelect } from '@base-ui/react/select';
import { ChevronDown } from 'lucide-react';

import { cn } from '../lib/cn';

export interface SelectOption {
  value: string;
  label: string;
}

export interface SelectProps {
  /** Controlled value. Use `null` when nothing is selected. */
  value?: string | null;
  /** Initial uncontrolled value. */
  defaultValue?: string | null;
  /** Called with the new value (string) when the user picks an option. */
  onValueChange?: (value: string | null) => void;
  options?: SelectOption[];
  /** Optional children rendered as <Select.Item> elements when `options` is not provided. */
  children?: React.ReactNode;
  className?: string;
  id?: string;
  name?: string;
  disabled?: boolean;
  'aria-label'?: string;
}

export function Select({
  value,
  defaultValue,
  onValueChange,
  options,
  children,
  className,
  id,
  name,
  disabled,
  'aria-label': ariaLabel,
}: SelectProps) {
  return (
    <BaseSelect.Root
      id={id}
      name={name}
      disabled={disabled}
      value={value ?? null}
      defaultValue={defaultValue ?? null}
      onValueChange={(next) => onValueChange?.((next as string | null) ?? null)}
      items={options?.map((opt) => ({ value: opt.value, label: opt.label }))}
    >
      <BaseSelect.Trigger
        aria-label={ariaLabel}
        className={cn(
          'focus-visible:ring-primary-500 flex h-10 w-full items-center justify-between gap-2 rounded-lg border border-white/60 bg-white/50 px-3 py-2 text-sm text-neutral-900 backdrop-blur-md transition-all',
          'hover:bg-white/70 dark:hover:bg-slate-800/70',
          'focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-offset-2 focus-visible:ring-offset-transparent',
          'disabled:cursor-not-allowed disabled:opacity-50',
          'dark:border-white/10 dark:bg-slate-800/50 dark:text-slate-100',
          className
        )}
      >
        <BaseSelect.Value placeholder="—" className="min-w-0 truncate" />
        <BaseSelect.Icon className="pointer-events-none flex shrink-0 items-center">
          <ChevronDown className="h-4 w-4 text-neutral-400" aria-hidden="true" />
        </BaseSelect.Icon>
      </BaseSelect.Trigger>
      <BaseSelect.Portal>
        <BaseSelect.Positioner sideOffset={4} className="z-50">
          <BaseSelect.Popup
            className={cn(
              'shadow-glass dark:shadow-glass-dark max-h-60 min-w-[var(--anchor-width)] overflow-auto rounded-xl border border-white/60 bg-white/90 py-1 backdrop-blur-xl dark:border-white/10 dark:bg-slate-900/90',
              'transition-opacity duration-150 data-[ending-style]:opacity-0 data-[starting-style]:opacity-0'
            )}
          >
            {options
              ? options.map((option) => (
                  <BaseSelect.Item
                    key={option.value}
                    value={option.value}
                    className={cn(
                      'focus-visible:ring-primary-500 relative flex w-full cursor-pointer select-none items-center rounded-sm py-1.5 pl-8 pr-2 text-sm',
                      'data-[selected]:bg-primary-50 data-[selected]:text-primary-900 outline-none',
                      'dark:data-[selected]:bg-primary-900/40 dark:data-[selected]:text-primary-100',
                      'data-[highlighted]:bg-neutral-100 dark:data-[highlighted]:bg-neutral-800'
                    )}
                  >
                    <BaseSelect.ItemIndicator
                      className="absolute left-2 flex h-4 w-4 items-center justify-center"
                      aria-hidden="true"
                    >
                      <span className="bg-primary-600 h-1.5 w-1.5 rounded-full" />
                    </BaseSelect.ItemIndicator>
                    <BaseSelect.ItemText>{option.label}</BaseSelect.ItemText>
                  </BaseSelect.Item>
                ))
              : children}
          </BaseSelect.Popup>
        </BaseSelect.Positioner>
      </BaseSelect.Portal>
    </BaseSelect.Root>
  );
}
