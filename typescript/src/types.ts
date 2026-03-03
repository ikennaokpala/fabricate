/**
 * Factory type definitions for BDD test data management.
 * Used by the BDD executor to create and track test entities
 * via the backend's __test__ endpoints.
 */

export type ResetScope = 'all' | 'users' | 'rides' | 'payments';

export interface SeedEntities {
  rider_id?: string;
  rider_email?: string;
  driver_id?: string;
  driver_email?: string;
  ride_id?: string;
  payment_method_id?: string;
  payment_intent_id?: string;
  incident_id?: string;
  vehicle_id?: string;
  auth_token?: string;
}

export interface CreatedEntity {
  id: string;
  type: string;
  email?: string;
  authToken?: string;
  attributes: Record<string, any>;
}

export interface FactoryContext {
  entities: CreatedEntity[];
  byType: Map<string, CreatedEntity[]>;
  authTokens: Map<string, string>;
}

export interface FactoryDefinition {
  entityName: string;
  scenario?: string;
  authRole?: 'rider' | 'driver' | 'admin';
  traits?: Record<string, any>;
  afterCreate?: (entity: CreatedEntity, ctx: FactoryContext) => Promise<void>;
}
