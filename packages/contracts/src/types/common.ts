export type UUID = string;

export interface Timestamps {
  createdAt: string; // ISO 8601
  updatedAt: string;
}

export interface PaginatedResponse<T> {
  items: T[];
  total: number;
  offset: number;
  limit: number;
}

export type ApiResponse<T> = {
  data: T;
  error?: never;
} | {
  data?: never;
  error: ApiError;
};

export interface ApiError {
  code: string;
  message: string;
  details?: Record<string, unknown>;
}
