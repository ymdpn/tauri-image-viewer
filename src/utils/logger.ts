import { invoke } from '@tauri-apps/api/tauri';

export const logInfo = (...args: any[]) => {
  console.log(...args);
  invoke('log_info', { message: JSON.stringify(args) });
};

export const logError = (...args: any[]) => {
  console.error(...args);
  invoke('log_error', { message: JSON.stringify(args) });
};