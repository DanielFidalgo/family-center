import { UUID, Timestamps } from './common';

export interface SyncCheckpoint extends Timestamps {
  id: UUID;
  calendarSourceId: UUID;
  syncToken?: string;     // Google incremental sync token
  fullSyncAt?: string;    // last full sync
  nextPageToken?: string; // pagination resume
}

export interface SyncRunRequest {
  calendarSourceIds?: UUID[];  // null = sync all selected
  forceFullSync?: boolean;
}

export interface SyncRunResponse {
  synced: number;
  created: number;
  updated: number;
  errors: Array<{ calendarSourceId: UUID; error: string }>;
}
