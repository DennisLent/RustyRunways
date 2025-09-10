import { describe, it, expect } from 'vitest'
import { isTauri } from './tauri'

type TestWindow = {
  __TAURI_IPC__?: () => void
  __TAURI__?: unknown
  __TAURI_INTERNALS__?: unknown
}

function withMockWindow(win: TestWindow, fn: () => void) {
  const g = globalThis as { window?: TestWindow }
  const prev = g.window
  g.window = win
  try {
    fn()
  } finally {
    g.window = prev
  }
}

describe('isTauri', () => {
  it('returns false when no window present', () => {
    // In Node environment, window is typically undefined
    // Ensure function handles it gracefully
    expect(isTauri()).toBe(false)
  })

  it('detects __TAURI_IPC__ global', () => {
    withMockWindow({ __TAURI_IPC__: () => {} }, () => {
      expect(isTauri()).toBe(true)
    })
  })

  it('detects __TAURI__ global', () => {
    withMockWindow({ __TAURI__: {} }, () => {
      expect(isTauri()).toBe(true)
    })
  })

  it('detects __TAURI_INTERNALS__ global', () => {
    withMockWindow({ __TAURI_INTERNALS__: {} }, () => {
      expect(isTauri()).toBe(true)
    })
  })
})
