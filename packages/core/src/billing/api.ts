/**
 * Billing / usage API placeholder per spec 12 section 4.3.
 *
 * Real implementation lands in v2.0.0 alongside Relay support. The signatures
 * stay stable so the UI can wire against them today.
 */

export type Plan = 'free' | 'pro' | 'enterprise';

export interface UsageInfo {
  plan: Plan;
  relayBytesUsed: number;
  relayBytesLimit: number;
  relayDurationUsed: number;
  relayDurationLimit: number;
}

export const FREE_PLAN_LIMIT: UsageInfo = {
  plan: 'free',
  relayBytesUsed: 0,
  relayBytesLimit: 100 * 1024 * 1024,
  relayDurationUsed: 0,
  relayDurationLimit: 0,
};

export async function getUsage(): Promise<UsageInfo> {
  return FREE_PLAN_LIMIT;
}

export async function upgradePlan(plan: 'pro' | 'enterprise'): Promise<string> {
  return `https://dropvoice.app/upgrade?plan=${plan}`;
}
