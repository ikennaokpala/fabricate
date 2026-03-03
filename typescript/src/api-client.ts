/**
 * API client for backend test endpoints.
 * Communicates with /__test__/* endpoints for entity creation/reset
 * and /cleanup, /seed for bulk operations.
 */

import { ResetScope, SeedEntities } from './types.js';

const API_BASE = process.env.SENTINEL_BACKEND_URL ?? 'http://localhost:8080/api/v1/test';
const TEST_KEY = process.env.TEST_API_KEY ?? 'test-key';

async function apiCall<T>(method: string, path: string, body?: unknown): Promise<T | null> {
  try {
    const resp = await fetch(`${API_BASE}${path}`, {
      method,
      headers: {
        'Content-Type': 'application/json',
        'X-Test-Key': TEST_KEY,
      },
      ...(body ? { body: JSON.stringify(body) } : {}),
    });
    if (!resp.ok) {
      console.error(`[Factory API] ${method} ${path} → ${resp.status}`);
      return null;
    }
    const text = await resp.text();
    return text ? JSON.parse(text) : ({} as T);
  } catch (err) {
    console.error(`[Factory API] ${method} ${path} failed:`, err);
    return null;
  }
}

export async function resetDatabase(scope: ResetScope): Promise<boolean> {
  const result = await apiCall<Record<string, unknown>>('POST', '/__test__/reset', { scope });
  return result !== null;
}

export async function seedScenario(
  scenario: string,
  seed?: number,
): Promise<{ entities: SeedEntities; message: string } | null> {
  return apiCall('POST', '/__test__/seed', {
    scenario,
    seed: seed ?? Date.now() % 100000,
  });
}

export async function createTestUser(
  role: 'rider' | 'driver' | 'admin',
  seed?: number,
): Promise<{ user_id: string; email: string; full_name: string; role: string; auth_token: string } | null> {
  return apiCall('POST', '/__test__/users', {
    role,
    seed: seed ?? Date.now() % 100000,
  });
}

export async function createTestDriver(
  seed?: number,
  withVehicle = true,
  verified = false,
): Promise<{ user_id: string; email: string; full_name: string; role: string; auth_token: string; vehicle_id?: string } | null> {
  return apiCall('POST', '/__test__/drivers', {
    seed: seed ?? Date.now() % 100000,
    with_vehicle: withVehicle,
    verified,
  });
}

export async function createTestRide(
  riderId: string,
  driverId?: string,
  status = 'requested',
  seed?: number,
): Promise<{ ride_id: string; status: string } | null> {
  return apiCall('POST', '/__test__/rides', {
    rider_id: riderId,
    driver_id: driverId,
    status,
    seed: seed ?? Date.now() % 100000,
  });
}

export async function createTestPayment(
  userId: string,
  amountCents: number,
  seed?: number,
): Promise<{ payment_id: string; amount_cents: number; status: string } | null> {
  return apiCall('POST', '/__test__/payments', {
    user_id: userId,
    amount_cents: amountCents,
    seed: seed ?? Date.now() % 100000,
  });
}

export async function fullCleanup(): Promise<boolean> {
  const result = await apiCall<Record<string, unknown>>('POST', '/cleanup');
  return result !== null;
}

export async function bulkSeed(
  options?: { reset?: boolean; users?: number; drivers?: number; rides?: number },
): Promise<Record<string, unknown> | null> {
  return apiCall('POST', '/seed', options ?? {});
}

export async function healthCheck(): Promise<boolean> {
  try {
    const healthUrl = API_BASE.replace(/\/test$/, '/health');
    const resp = await fetch(healthUrl, {
      method: 'GET',
      headers: { 'X-Test-Key': TEST_KEY },
    });
    return resp.ok;
  } catch {
    return false;
  }
}
