import { type ClassValue, clsx } from 'clsx';
import { twMerge } from 'tailwind-merge';

/**
 * Merge Tailwind CSS classes with conditional logic.
 *
 * Combines `clsx` (conditional class joining) with `tailwind-merge`
 * (conflict resolution so the last utility wins, e.g. `p-2 p-4` -> `p-4`).
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}
