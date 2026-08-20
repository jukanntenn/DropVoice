/**
 * Type-safe translation key utilities.
 *
 * The keys are derived from the canonical English bundle so that any missing
 * translation in another language surfaces at compile time when used through
 * the `TranslationKey` alias.
 */

export interface TranslationBundle {
  common: Record<string, unknown>;
  devices: Record<string, unknown>;
  errors: Record<string, unknown>;
  landing: Record<string, unknown>;
  settings: Record<string, unknown>;
}

/**
 * Recursively flattens a nested object into dot-notation keys.
 *
 * Example: `{ app: { name: 'x' } }` becomes `'app.name'`.
 */
export type FlattenKeys<T, Prefix extends string = ''> = T extends object
  ? {
      [K in keyof T & string]: T[K] extends object
        ? FlattenKeys<T[K], `${Prefix}${K}.`>
        : `${Prefix}${K}`;
    }[keyof T & string]
  : never;
