/**
 * Factory class that orchestrates test entity creation
 * via backend __test__ endpoints. Tracks all created entities
 * in a FactoryContext for cross-step reference in BDD scenarios.
 */

import { CreatedEntity, FactoryContext } from './types.js';
import * as api from './api-client.js';

export class Factory {
  private context: FactoryContext;

  constructor() {
    this.context = { entities: [], byType: new Map(), authTokens: new Map() };
  }

  async create(type: string, traitOrOverrides?: string | Record<string, any>): Promise<CreatedEntity> {
    const trait = typeof traitOrOverrides === 'string' ? traitOrOverrides : undefined;
    const overrides = typeof traitOrOverrides === 'object' ? traitOrOverrides : {};

    switch (type) {
      case 'rider':
        return this.createUser('rider');
      case 'driver':
        return this.createDriver();
      case 'admin':
        return this.createUser('admin');
      case 'trip':
      case 'ride':
        return this.createRide(trait, overrides);
      case 'booking':
        return this.createRide(trait, overrides);
      case 'vehicle':
        return this.createVehicle();
      case 'payment':
        return this.createPayment(overrides);
      case 'completed_ride':
        return this.createFromScenario('complete_ride', 'completed_ride');
      case 'sos_emergency':
        return this.createFromScenario('sos_emergency', 'sos_emergency');
      default:
        throw new Error(`[Factory] Unknown entity type: ${type}`);
    }
  }

  getEntity(type: string): CreatedEntity | undefined {
    const list = this.context.byType.get(type);
    return list ? list[list.length - 1] : undefined;
  }

  getAuthToken(role: string): string | undefined {
    return this.context.authTokens.get(role);
  }

  async reset(): Promise<void> {
    await api.resetDatabase('all');
    this.context = { entities: [], byType: new Map(), authTokens: new Map() };
  }

  async teardown(): Promise<void> {
    await api.resetDatabase('all');
  }

  private store(entity: CreatedEntity): void {
    this.context.entities.push(entity);
    const list = this.context.byType.get(entity.type) ?? [];
    list.push(entity);
    this.context.byType.set(entity.type, list);
    if (entity.authToken) {
      this.context.authTokens.set(entity.type, entity.authToken);
    }
  }

  private async createUser(role: 'rider' | 'admin'): Promise<CreatedEntity> {
    const result = await api.createTestUser(role);
    if (!result) throw new Error(`[Factory] Failed to create ${role}`);

    const entity: CreatedEntity = {
      id: result.user_id,
      type: role,
      email: result.email,
      authToken: result.auth_token,
      attributes: { full_name: result.full_name, role: result.role },
    };
    this.store(entity);
    return entity;
  }

  private async createDriver(): Promise<CreatedEntity> {
    const result = await api.createTestDriver(undefined, true, false);
    if (!result) throw new Error('[Factory] Failed to create driver');

    const entity: CreatedEntity = {
      id: result.user_id,
      type: 'driver',
      email: result.email,
      authToken: result.auth_token,
      attributes: {
        full_name: result.full_name,
        role: result.role,
        vehicle_id: result.vehicle_id,
      },
    };
    this.store(entity);
    return entity;
  }

  private async createRide(
    trait?: string,
    overrides?: Record<string, any>,
  ): Promise<CreatedEntity> {
    const scenario = trait === 'completed' ? 'complete_ride' : 'rider_book_ride';
    const result = await api.seedScenario(scenario);
    if (!result) throw new Error(`[Factory] Failed to seed scenario: ${scenario}`);

    const ents = result.entities;

    // Store side-effect entities (rider, driver) if created by the scenario
    if (ents.rider_id) {
      const riderEntity: CreatedEntity = {
        id: ents.rider_id,
        type: 'rider',
        email: ents.rider_email,
        authToken: ents.auth_token,
        attributes: {},
      };
      this.store(riderEntity);
    }
    if (ents.driver_id) {
      const driverEntity: CreatedEntity = {
        id: ents.driver_id,
        type: 'driver',
        email: ents.driver_email,
        attributes: {},
      };
      this.store(driverEntity);
    }

    const rideEntity: CreatedEntity = {
      id: ents.ride_id ?? '',
      type: trait === 'completed' ? 'completed_ride' : 'ride',
      attributes: { ...ents, ...overrides },
    };
    this.store(rideEntity);
    return rideEntity;
  }

  private async createVehicle(): Promise<CreatedEntity> {
    const result = await api.seedScenario('driver_onboard');
    if (!result) throw new Error('[Factory] Failed to seed driver_onboard for vehicle');

    const entity: CreatedEntity = {
      id: result.entities.vehicle_id ?? '',
      type: 'vehicle',
      attributes: { ...result.entities },
    };
    this.store(entity);
    return entity;
  }

  private async createPayment(overrides?: Record<string, any>): Promise<CreatedEntity> {
    let userId = this.getEntity('rider')?.id;
    if (!userId) {
      const rider = await this.createUser('rider');
      userId = rider.id;
    }

    const amountCents = overrides?.amount_cents ?? 2500;
    const result = await api.createTestPayment(userId, amountCents);
    if (!result) throw new Error('[Factory] Failed to create payment');

    const entity: CreatedEntity = {
      id: result.payment_id,
      type: 'payment',
      attributes: { amount_cents: result.amount_cents, status: result.status, user_id: userId },
    };
    this.store(entity);
    return entity;
  }

  private async createFromScenario(scenario: string, entityType: string): Promise<CreatedEntity> {
    const result = await api.seedScenario(scenario);
    if (!result) throw new Error(`[Factory] Failed to seed scenario: ${scenario}`);

    const ents = result.entities;
    const entity: CreatedEntity = {
      id: ents.ride_id ?? ents.incident_id ?? '',
      type: entityType,
      attributes: { ...ents },
    };
    this.store(entity);
    return entity;
  }
}
