# Family Center Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development to implement this plan task-by-task.

**Goal:** Build a wall-mounted family scheduler monorepo with Rust backend and Ionic React mobile app.

**Architecture:** Axum+SQLx Rust backend with PostgreSQL; Ionic React + Capacitor Android app with SQLite local cache; shared TypeScript contracts package.

**Tech Stack:** Rust/Axum/SQLx/PostgreSQL, TypeScript/Ionic React/Capacitor, TanStack Query, Zustand, SQLite

---

## Tasks
1. Monorepo scaffold (done)
2. Shared contracts package
3. Rust backend: Cargo + migrations
4. Rust backend: models + API handlers
5. Rust backend: Google OAuth + sync + dedupe
6. Rust backend: tests
7. Mobile: scaffold + structure
8. Mobile: Day/Week board screens
9. Mobile: Setup/management screens
10. Docs
