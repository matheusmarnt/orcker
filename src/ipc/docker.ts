import { invoke } from '@tauri-apps/api/core'

// AppError type — will be replaced by auto-generated bindings.ts in Plan 03
export interface AppError {
  kind: 'DockerUnavailable' | 'DockerApi' | 'DockerPermission' | 'ContainerNotFound' | 'Internal'
  message: string
}

export type IpcResult<T> = { ok: true; value: T } | { ok: false; error: AppError }

export async function safe<T>(p: Promise<T>): Promise<IpcResult<T>> {
  try {
    return { ok: true, value: await p }
  } catch (e) {
    return { ok: false, error: e as AppError }
  }
}

export async function getDockerVersion(): Promise<string> {
  return invoke<string>('get_docker_version')
}

export async function listContainers(): Promise<unknown[]> {
  return invoke<unknown[]>('list_containers')
}
