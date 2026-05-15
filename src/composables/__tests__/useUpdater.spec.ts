import { describe, it, expect, vi, beforeEach } from 'vitest'

// vi.mock is hoisted — factory must not reference outer const variables (Vitest TDZ)
vi.mock('@tauri-apps/plugin-updater', () => ({
  check: vi.fn(),
}))

vi.mock('@tauri-apps/plugin-process', () => ({
  relaunch: vi.fn(),
}))

vi.mock('vue-sonner', () => ({
  toast: vi.fn(),
}))

import { checkForUpdate } from '../useUpdater'
import { check, type Update } from '@tauri-apps/plugin-updater'
import { toast } from 'vue-sonner'

const mockCheck = vi.mocked(check)
const mockToast = vi.mocked(toast)

describe('useUpdater', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('check() returns null when no update available — no toast shown (R-M7.5)', async () => {
    mockCheck.mockResolvedValue(null)

    await checkForUpdate()

    expect(mockToast).not.toHaveBeenCalled()
  })

  it('shows Sonner toast with version and Install action when update is available', async () => {
    const fakeUpdate = {
      version: '0.5.0',
      downloadAndInstall: vi.fn().mockResolvedValue(undefined),
    } as unknown as Update
    mockCheck.mockResolvedValue(fakeUpdate)

    await checkForUpdate()

    expect(mockToast).toHaveBeenCalledOnce()
    const [message, options] = mockToast.mock.calls[0]
    expect(message).toContain('0.5.0')
    expect(options).toMatchObject({
      duration: Infinity,
      action: expect.objectContaining({ label: 'Install' }),
    })
  })
})
