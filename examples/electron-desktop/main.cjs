'use strict';

const path = require('node:path');
const { app, BrowserWindow, ipcMain } = require('electron');
const { parseBytes } = require('@harrisonwang/spoor');

const MAX_PARSE_BYTES = 64 * 1024 * 1024;

function createWindow() {
  const window = new BrowserWindow({
    width: 1180,
    height: 800,
    minWidth: 820,
    minHeight: 620,
    backgroundColor: '#171914',
    titleBarStyle: process.platform === 'darwin' ? 'hiddenInset' : 'default',
    webPreferences: {
      preload: path.join(__dirname, 'preload.cjs'),
      contextIsolation: true,
      nodeIntegration: false,
      sandbox: true,
    },
  });

  window.loadFile('index.html');
}

ipcMain.handle('spoor:parse', (_event, input) => {
  try {
    return {
      ok: true,
      result: parseBytes(Buffer.from(input.bytes), {
        sourceName: input.sourceName,
        contentType: input.contentType || undefined,
        maxParseBytes: MAX_PARSE_BYTES,
      }),
    };
  } catch (error) {
    return {
      ok: false,
      error: {
        code: error.code || 'parse_failed',
        reason: error.reason || error.message || String(error),
        hint: error.hint || '',
        recoverable: Boolean(error.recoverable),
        stage: error.stage,
      },
    };
  }
});

app.whenReady().then(() => {
  createWindow();
  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});
