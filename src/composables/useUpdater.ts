import { check } from '@tauri-apps/plugin-updater'
import { relaunch } from '@tauri-apps/plugin-process'
import { toast } from 'vue-sonner'

export async function checkForUpdate(): Promise<void> {
  try {
    const update = await check()
    if (!update) return
    toast(`Update available: v${update.version}`, {
      duration: Infinity,
      action: {
        label: 'Install',
        onClick: async () => {
          await update.downloadAndInstall()
          await relaunch()
        },
      },
    })
  } catch {
    // Updater check failure is non-fatal (e.g. no network, dev mode)
    // Do not show error to user
  }
}
