export interface BootstrapRequest {
  householdName?: string;
}

export interface BootstrapResponse {
  householdId: string;
  token: string;             // JWT for subsequent requests
  isNew: boolean;
}
