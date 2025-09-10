import { describe, it, expect } from 'vitest'
import { isTauri } from './tauri'

describe('isTauri', () => {
  it('returns false when no window present', () => {
    // In Node environment, window is typically undefined
    // Ensure function handles it gracefully
    expect(isTauri()).toBe(false)
  })

  it('detects __TAURI_IPC__ global', () => {
    const prev = (globalThis as any).window
    ;(globalThis as any).window = { __TAURI_IPC__: () => {} }
    try {
      expect(isTauri()).toBe(true)
    } finally {
      ;(globalThis as any).window = prev
    }
  })

  it('detects __TAURI__ global', () => {
    const prev = (globalThis as any).window
    ;(globalThis as any).window = { __TAURI__: {} }
    try {
      expect(isTauri()).toBe(true)
    } finally {
      ;(globalThis as any).window = prev
    }
  })

  it('detects __TAURI_INTERNALS__ global', () => {
    const prev = (globalThis as any).window
    ;(globalThis as any).window = { __TAURI_INTERNALS__: {} }
    try {
      expect(isTauri()).toBe(true)
    } finally {
      ;(globalThis as any).window = prev
    }
  })
})
