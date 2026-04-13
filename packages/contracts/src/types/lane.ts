import { UUID, Timestamps } from './common';

export type LaneTarget = 'person' | 'shared';

export interface LaneAssignmentRule extends Timestamps {
  id: UUID;
  householdId: UUID;
  calendarSourceId?: UUID;   // match by calendar
  emailPattern?: string;     // match organizer/attendee email (glob)
  personId?: UUID;           // null = shared
  laneTarget: LaneTarget;
  priority: number;          // lower = higher priority
}

export interface LinkAccountRequest {
  googleAccountId: UUID;
  personId?: UUID;           // null = shared lane
}

export interface UnlinkAccountRequest {
  googleAccountId: UUID;
  personId?: UUID;           // null = shared lane
}

export interface LinkResult {
  rulesCreated: number;
}

export interface UnlinkResult {
  rulesDeleted: number;
}
