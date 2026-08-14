/**
 * PageContainer — centered max-width container (spec 07 section 2.3).
 */
import type { ReactNode } from 'react';

import { cn } from '../lib/cn';

export interface PageContainerProps {
  children: ReactNode;
  maxWidth?: 'sm' | 'md' | 'lg' | 'xl' | '2xl' | 'full';
  className?: string;
}

const maxWidthClasses: Record<NonNullable<PageContainerProps['maxWidth']>, string> = {
  sm: 'max-w-sm',
  md: 'max-w-md',
  lg: 'max-w-lg',
  xl: 'max-w-xl',
  '2xl': 'max-w-2xl',
  full: 'max-w-full',
};

export function PageContainer({ children, maxWidth = 'lg', className }: PageContainerProps) {
  return (
    <main className={cn('mx-auto w-full px-4 py-6', maxWidthClasses[maxWidth], className)}>
      {children}
    </main>
  );
}
