import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { api } from './client';
import type {
  Person, CreatePersonRequest, UpdatePersonRequest,
  GoogleAccountPublic, CalendarSource,
  LocalActivity, CreateActivityRequest, UpdateActivityRequest, CompleteActivityRequest, ActivityCompletion,
  Settings, UpdateSettingsRequest,
  ScheduleResponse, SyncRunRequest, SyncRunResponse,
  LaneAssignmentRule, LinkAccountRequest, UnlinkAccountRequest,
  LinkResult, UnlinkResult,
} from '@family-center/contracts';

// ---- People ----
export const PEOPLE_KEY = ['people'] as const;

export function usePeople() {
  return useQuery({
    queryKey: PEOPLE_KEY,
    queryFn: () => api.get<Person[]>('/people'),
  });
}

export function useCreatePerson() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: CreatePersonRequest) => api.post<Person>('/people', body),
    onSuccess: () => qc.invalidateQueries({ queryKey: PEOPLE_KEY }),
  });
}

export function useUpdatePerson() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...body }: UpdatePersonRequest & { id: string }) =>
      api.patch<Person>(`/people/${id}`, body),
    onSuccess: () => qc.invalidateQueries({ queryKey: PEOPLE_KEY }),
  });
}

// ---- Google Accounts ----
export const GOOGLE_ACCOUNTS_KEY = ['google-accounts'] as const;

export function useGoogleAccounts() {
  return useQuery({
    queryKey: GOOGLE_ACCOUNTS_KEY,
    queryFn: () => api.get<GoogleAccountPublic[]>('/google/accounts'),
  });
}

export function useCalendars(accountId: string) {
  return useQuery({
    queryKey: ['calendars', accountId],
    queryFn: () => api.get<CalendarSource[]>(`/google/accounts/${accountId}/calendars`),
    enabled: !!accountId,
  });
}

// ---- Schedule ----
export function useSchedule(start: string, end: string) {
  return useQuery({
    queryKey: ['schedule', start, end],
    queryFn: () => api.get<ScheduleResponse>(`/schedule?start=${start}&end=${end}`),
    staleTime: 2 * 60 * 1000, // 2 min
  });
}

// ---- Activities ----
export const ACTIVITIES_KEY = ['activities'] as const;

export function useActivities() {
  return useQuery({
    queryKey: ACTIVITIES_KEY,
    queryFn: () => api.get<LocalActivity[]>('/activities'),
  });
}

export function useCreateActivity() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: CreateActivityRequest) => api.post<LocalActivity>('/activities', body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ACTIVITIES_KEY });
      qc.invalidateQueries({ queryKey: ['schedule'] });
    },
  });
}

export function useUpdateActivity() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...body }: UpdateActivityRequest & { id: string }) =>
      api.patch<LocalActivity>(`/activities/${id}`, body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ACTIVITIES_KEY });
      qc.invalidateQueries({ queryKey: ['schedule'] });
    },
  });
}

// ---- Settings ----
export const SETTINGS_KEY = ['settings'] as const;

export function useSettings() {
  return useQuery({
    queryKey: SETTINGS_KEY,
    queryFn: () => api.get<Settings>('/settings'),
  });
}

export function useUpdateSettings() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: UpdateSettingsRequest) => api.patch<Settings>('/settings', body),
    onSuccess: () => qc.invalidateQueries({ queryKey: SETTINGS_KEY }),
  });
}

// ---- Sync ----
export function useRunSync() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: SyncRunRequest) => api.post<SyncRunResponse>('/sync/run', body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['schedule'] });
    },
  });
}

export function useConnectGoogleStart() {
  return useMutation({
    mutationFn: () => api.post<{ authUrl: string; state: string }>('/google/connect/start'),
  });
}

// ---- Activity Completions ----
export function useCompleteActivity() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...body }: CompleteActivityRequest & { id: string }) =>
      api.post<ActivityCompletion>(`/activities/${id}/complete`, body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['schedule'] });
    },
  });
}

export function useUncompleteActivity() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: ({ id, ...body }: CompleteActivityRequest & { id: string }) =>
      api.post<ActivityCompletion[]>(`/activities/${id}/uncomplete`, body),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['schedule'] });
    },
  });
}

// ---- Claim Tokens ----
export function useCreateClaimToken() {
  return useMutation({
    mutationFn: (personId: string) =>
      api.post<{ token: string; expiresAt: string; claimUrl: string }>(`/people/${personId}/claim-token`, {}),
  });
}

// ---- Lane Assignment Rules ----
export const LANE_RULES_KEY = ['lane-rules'] as const;

export function useLaneRules() {
  return useQuery({
    queryKey: LANE_RULES_KEY,
    queryFn: () => api.get<LaneAssignmentRule[]>('/lane-rules'),
  });
}

export function useLinkAccount() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: LinkAccountRequest) =>
      api.post<LinkResult>('/lane-rules/link-account', body),
    onSuccess: () => qc.invalidateQueries({ queryKey: LANE_RULES_KEY }),
  });
}

export function useUnlinkAccount() {
  const qc = useQueryClient();
  return useMutation({
    mutationFn: (body: UnlinkAccountRequest) =>
      api.post<UnlinkResult>('/lane-rules/unlink-account', body),
    onSuccess: () => qc.invalidateQueries({ queryKey: LANE_RULES_KEY }),
  });
}
