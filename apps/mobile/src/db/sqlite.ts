/**
 * SQLite local cache for offline-first schedule data.
 *
 * Strategy:
 * - After a successful sync, write schedule data into SQLite
 * - On startup, read from SQLite immediately (instant display)
 * - Sync in background; update SQLite on completion
 *
 * Tables mirror the server response shapes needed for display.
 */

import { Capacitor } from '@capacitor/core';
import type { ScheduleResponse } from '@family-center/contracts';

let db: any = null;
let dbInitialized = false;

const SCHEMA = `
  CREATE TABLE IF NOT EXISTS schedule_cache (
    cache_key TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    cached_at INTEGER NOT NULL
  );

  CREATE TABLE IF NOT EXISTS people_cache (
    id TEXT PRIMARY KEY,
    data TEXT NOT NULL,
    cached_at INTEGER NOT NULL
  );

  CREATE TABLE IF NOT EXISTS settings_cache (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    cached_at INTEGER NOT NULL
  );
`;

export async function initDb(): Promise<void> {
  if (dbInitialized) return;

  if (!Capacitor.isNativePlatform()) {
    // In web/dev mode, use localStorage as a simple fallback
    dbInitialized = true;
    return;
  }

  try {
    const { CapacitorSQLite, SQLiteConnection } = await import('@capacitor-community/sqlite');
    const sqlite = new SQLiteConnection(CapacitorSQLite);
    db = await sqlite.createConnection('family_center', false, 'no-encryption', 1, false);
    await db.open();
    await db.execute(SCHEMA);
    dbInitialized = true;
  } catch (e) {
    console.warn('SQLite init failed, falling back to localStorage', e);
    dbInitialized = true;
  }
}

// ---- Schedule cache ----

export async function cacheSchedule(key: string, data: ScheduleResponse): Promise<void> {
  const json = JSON.stringify(data);
  const now = Date.now();

  if (db) {
    await db.run(
      'INSERT OR REPLACE INTO schedule_cache (cache_key, data, cached_at) VALUES (?, ?, ?)',
      [key, json, now]
    );
  } else {
    localStorage.setItem(`fc:schedule:${key}`, JSON.stringify({ data: json, cachedAt: now }));
  }
}

export async function getCachedSchedule(key: string): Promise<ScheduleResponse | null> {
  const maxAge = 24 * 60 * 60 * 1000; // 24 hours

  if (db) {
    const result = await db.query(
      'SELECT data, cached_at FROM schedule_cache WHERE cache_key = ?',
      [key]
    );
    const row = result?.values?.[0];
    if (!row) return null;
    if (Date.now() - row.cached_at > maxAge) return null;
    return JSON.parse(row.data);
  } else {
    const raw = localStorage.getItem(`fc:schedule:${key}`);
    if (!raw) return null;
    const { data, cachedAt } = JSON.parse(raw);
    if (Date.now() - cachedAt > maxAge) return null;
    return JSON.parse(data);
  }
}

// ---- People cache ----

export async function cachePeople(people: any[]): Promise<void> {
  const now = Date.now();

  if (db) {
    for (const person of people) {
      await db.run(
        'INSERT OR REPLACE INTO people_cache (id, data, cached_at) VALUES (?, ?, ?)',
        [person.id, JSON.stringify(person), now]
      );
    }
  } else {
    localStorage.setItem('fc:people', JSON.stringify({ data: people, cachedAt: now }));
  }
}

export async function getCachedPeople(): Promise<any[] | null> {
  if (db) {
    const result = await db.query('SELECT data FROM people_cache ORDER BY data');
    return result?.values?.map((r: any) => JSON.parse(r.data)) ?? null;
  } else {
    const raw = localStorage.getItem('fc:people');
    if (!raw) return null;
    return JSON.parse(raw).data;
  }
}

// ---- Settings cache ----

export async function cacheSetting(key: string, value: unknown): Promise<void> {
  const now = Date.now();
  if (db) {
    await db.run(
      'INSERT OR REPLACE INTO settings_cache (key, value, cached_at) VALUES (?, ?, ?)',
      [key, JSON.stringify(value), now]
    );
  } else {
    localStorage.setItem(`fc:settings:${key}`, JSON.stringify({ value, cachedAt: now }));
  }
}

export async function getCachedSetting<T>(key: string): Promise<T | null> {
  if (db) {
    const result = await db.query(
      'SELECT value FROM settings_cache WHERE key = ?',
      [key]
    );
    const row = result?.values?.[0];
    return row ? JSON.parse(row.value) : null;
  } else {
    const raw = localStorage.getItem(`fc:settings:${key}`);
    if (!raw) return null;
    return JSON.parse(raw).value;
  }
}
