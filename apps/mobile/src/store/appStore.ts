import { create } from 'zustand';
import { persist } from 'zustand/middleware';

interface AppState {
  householdId: string | null;
  token: string | null;
  isOnboarded: boolean;
  apiBaseUrl: string;

  // Actions
  setHousehold: (id: string, token: string) => void;
  setOnboarded: (value: boolean) => void;
  setApiBaseUrl: (url: string) => void;
  reset: () => void;
}

const DEFAULT_API_URL = import.meta.env.VITE_API_BASE_URL || 'http://localhost:3000';

export const useAppStore = create<AppState>()(
  persist(
    (set) => ({
      householdId: null,
      token: null,
      isOnboarded: false,
      apiBaseUrl: DEFAULT_API_URL,

      setHousehold: (id, token) => set({ householdId: id, token, isOnboarded: true }),
      setOnboarded: (value) => set({ isOnboarded: value }),
      setApiBaseUrl: (url) => set({ apiBaseUrl: url }),
      reset: () => set({ householdId: null, token: null, isOnboarded: false }),
    }),
    {
      name: 'family-center-app',
    }
  )
);
