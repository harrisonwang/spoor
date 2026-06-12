'use strict';

const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('spoorDesktop', {
  parseDocument(input) {
    return ipcRenderer.invoke('spoor:parse', input);
  },
});
