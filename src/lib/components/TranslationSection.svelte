<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open, save } from '@tauri-apps/plugin-dialog';
  import { open as openPath } from '@tauri-apps/plugin-shell';
  import { downloadDir } from '@tauri-apps/api/path';
  import { writeTextFile } from '@tauri-apps/plugin-fs';
  import ProgressBar from './ProgressBar.svelte';
  import FolderSelectorModal from './FolderSelectorModal.svelte';

  export let selectedMode: string;
  export let folderPath: string;
  export let isTranslating: boolean;
  export let progress: number;
  export let status: string;
  export let logs: string[];
  export let model: string;
  export let targetLang: string;
  export let font: string;
  export let textAlign: string;
  export let strokeDisabled: boolean;
  export let inpaintOnly: boolean;
  export let minFontSize: number;
  export let includedPaths: string[] = [];
  export let showFolderSelector: boolean = false;
  export let customSelectorRoot: string = "";
  export let isSelectingForMangadex: boolean = false;
  export let showAndroidFolderPicker: boolean = false;
  let showAndroidOutputPicker: boolean = false;
  let useCustomOutput: boolean = false;
  let outputFolder: string = "";

  async function selectFolder() {
    status = "Opening folder picker...";
    
    if (navigator.userAgent.toLowerCase().includes("android") || (window as any).__TAURI_MOBILE__) {
      try {
        let base = "/storage/emulated/0/Download";
        try {
          base = await downloadDir();
        } catch(e) {}
        
        customSelectorRoot = base;
        isSelectingForMangadex = false;
        showAndroidFolderPicker = true;
        status = "Select a folder...";
        return;
      } catch(e) {
        alert("Could not access storage: " + e);
      }
      return;
    }

    try {
      const selected = await open({
        directory: true,
        multiple: false,
      });
      status = "Ready";
      if (selected) {
        folderPath = selected as string;
        includedPaths = [];
      }
    } catch (e) {
      status = "Error: " + e;
      alert("Error opening folder picker: " + e);
    }
  }

  async function selectOutputFolder() {
    status = "Opening output folder picker...";
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Output Directory"
      });
      status = "Ready";
      if (selected) {
        outputFolder = selected as string;
      }
    } catch (e) {
       status = "Error: " + e;
    }
  }

  function openFolderSelector() {
    if (!folderPath) {
      alert("Please select a folder first.");
      return;
    }
    showFolderSelector = true;
  }

  function handleFolderSelection(paths: string[]) {
    includedPaths = paths;
    alert(`Selected ${paths.length} items to translate.`);
    showFolderSelector = false;
  }

  async function openOutputFolder() {
    const target = useCustomOutput && outputFolder ? outputFolder : 
                  (folderPath + (selectedMode === 'archive' ? '/archive_outputs' : '/translated'));
    try {
      await openPath(target); 
    } catch (e) {
      alert("Failed to open folder: " + e);
    }
  }

  async function startTranslation() {
    if (!folderPath) {
      alert("Please select a folder first.");
      return;
    }

    isTranslating = true;
    status = "Starting...";
    progress = 0;
    logs = ["Starting translation..."];

    try {
      await invoke('start_translation', { 
        folderPath, 
        model, 
        mode: selectedMode,
        targetLang,
        font,
        textAlign,
        strokeDisabled,
        inpaintOnly,
        minFontSize,
        outputFolder: useCustomOutput ? outputFolder : null,
        includedPaths: includedPaths.length > 0 ? includedPaths : null
      });
      status = "Completed!";
      logs = [...logs, "Translation Completed Successfully!"];
      progress = 100;
    } catch (e) {
      status = "Error: " + e;
      logs = [...logs, "Error: " + e];
    } finally {
      isTranslating = false;
    }
  }

  async function copyLogs() {
    try {
      await navigator.clipboard.writeText(logs.join('\n'));
      alert("Logs copied to clipboard!");
    } catch (e) {
      alert("Failed to copy logs: " + e);
    }
  }

  async function saveLogs() {
    try {
      const path = await save({
        filters: [{
          name: 'Log File',
          extensions: ['txt', 'log']
        }]
      });
      if (path) {
        await writeTextFile(path, logs.join('\n'));
        alert("Logs saved!");
      }
    } catch (e) {
      alert("Failed to save logs: " + e);
    }
  }
</script>

<div class="space-y-4">
  <div class="space-y-2">
    <div class="block text-xs font-semibold text-gray-500 uppercase tracking-widest ml-1">Source Folder</div>
    <div class="flex gap-2">
      <input 
        type="text" 
        bind:value={folderPath} 
        placeholder={selectedMode === 'archive' ? "/path/to/folder/with/archives" : "/path/to/images"}
        class="flex-1 p-2 border rounded bg-gray-50 dark:bg-gray-700 dark:border-gray-600 dark:text-white truncate"
        readonly
      />
      <button 
        on:click={selectFolder}
        class="bg-blue-600 text-white px-4 py-2 rounded hover:bg-blue-700 transition-colors text-sm font-medium"
      >
        Browse
      </button>
    </div>
  </div>

  {#if folderPath}
    <div class="flex justify-start">
      <button 
        on:click={openFolderSelector}
        class="text-sm text-blue-600 hover:text-blue-800 dark:text-blue-400 dark:hover:text-blue-300 flex items-center gap-1.5 bg-blue-50 dark:bg-blue-900/20 px-3 py-2 rounded-xl transition-all border border-blue-100 dark:border-blue-800 shadow-sm hover:shadow-md active:scale-95 group"
      >
        <span class="text-base group-hover:rotate-12 transition-transform">📂</span> 
        <span class="font-medium tracking-tight">Select specific subfolders/files</span>
      </button>
      {#if includedPaths.length > 0}
        <div class="ml-3 flex items-center gap-1 self-center bg-green-50 dark:bg-green-900/10 px-2 py-1 rounded-lg border border-green-100 dark:border-green-800 animate-in zoom-in duration-200">
            <span class="text-green-500 text-[10px]">●</span>
            <span class="text-[10px] font-bold text-green-600 dark:text-green-400 uppercase tracking-tighter">{includedPaths.length} items selected</span>
        </div>
        <button 
           on:click={() => includedPaths = []} 
           class="ml-2 text-[10px] text-gray-400 hover:text-red-500 transition-colors uppercase font-bold"
           title="Clear selection"
        >
          Clear
        </button>
      {/if}
    </div>
  {/if}

  <div class="bg-gray-100/50 dark:bg-gray-800/50 p-3 rounded-xl border border-gray-200 dark:border-gray-700 space-y-3">
    <div class="flex items-center justify-between">
      <div class="flex items-center gap-2">
        <div class="text-xs font-semibold text-gray-600 dark:text-gray-400 uppercase tracking-widest">Output Settings</div>
        {#if targetLang && !["en","ja","ko","tr","zh"].includes(targetLang)}
          <div class="group relative">
            <span class="text-[10px] bg-yellow-100 text-yellow-800 px-1.5 py-0.5 rounded border border-yellow-200 cursor-help">Custom Language Active</span>
            <div class="absolute left-0 bottom-full mb-2 w-48 p-2 bg-gray-900 text-white text-[10px] rounded shadow-xl opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-20">
              Files will be saved in the '{selectedMode === 'archive' ? 'archive_outputs' : 'translated'}' folder regardless of language name.
            </div>
          </div>
        {/if}
      </div>
      <label class="flex items-center gap-2 cursor-pointer group">
        <span class="text-[10px] text-gray-500 group-hover:text-blue-500 transition-colors uppercase font-bold">Custom path</span>
        <input type="checkbox" bind:checked={useCustomOutput} class="rounded text-blue-600 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600" />
      </label>
    </div>

    {#if useCustomOutput}
      <div class="flex gap-2 animate-in fade-in slide-in-from-top-1 duration-200">
        <input 
          type="text" 
          bind:value={outputFolder} 
          placeholder="Select custom output folder..."
          class="flex-1 p-1.5 text-xs border rounded bg-white dark:bg-gray-700 dark:border-gray-600 dark:text-white truncate"
          readonly
        />
        <button 
          on:click={selectOutputFolder}
          class="bg-gray-500 text-white px-3 py-1 rounded hover:bg-gray-600 transition-colors text-xs"
        >
          Select
        </button>
      </div>
    {/if}

    <div class="flex items-center gap-2 text-xs bg-white dark:bg-gray-900 p-2 rounded border border-gray-100 dark:border-gray-800">
      <span class="text-blue-500">📁</span>
      <span class="text-gray-400">Target:</span>
      <span class="text-gray-700 dark:text-gray-300 font-mono truncate">
        {#if useCustomOutput}
          {outputFolder || 'Please select a folder...'}
        {:else if folderPath}
          {folderPath}/{selectedMode === 'archive' ? 'archive_outputs' : 'translated'}
        {:else}
          Select a source folder first
        {/if}
      </span>
    </div>
  </div>


  {#if isTranslating}
    <ProgressBar {progress} {status} />
  {:else}
    <button 
      on:click={startTranslation}
      class="w-full bg-blue-600 text-white py-3 rounded-lg font-medium hover:bg-blue-700 transition-colors"
    >
      Start Translation
    </button>
    {#if status !== "Ready" || logs.length > 0}
      <div class="mt-4 border rounded bg-gray-50 dark:bg-gray-700 dark:border-gray-600 p-3">
        <div class="flex justify-between items-center mb-2">
          <span class="text-sm font-semibold text-gray-700 dark:text-gray-200">Status Log</span>
          <div class="flex gap-2">
            {#if status === "Completed!"}
              <button on:click={openOutputFolder} class="text-xs bg-green-100 text-green-800 border border-green-200 px-2 py-1 rounded hover:bg-green-200">Open Folder</button>
            {/if}
            <button on:click={copyLogs} class="text-xs bg-white dark:bg-gray-600 dark:text-white border dark:border-gray-500 px-2 py-1 rounded hover:bg-gray-100 dark:hover:bg-gray-500">Copy</button>
            <button on:click={saveLogs} class="text-xs bg-white dark:bg-gray-600 dark:text-white border dark:border-gray-500 px-2 py-1 rounded hover:bg-gray-100 dark:hover:bg-gray-500">Save</button>
          </div>
        </div>
        <div class="bg-white dark:bg-gray-800 dark:text-gray-300 border dark:border-gray-600 rounded p-2 h-32 overflow-y-auto text-xs font-mono whitespace-pre-wrap">
          {#each logs as log}
            <div>{log}</div>
          {/each}
          {#if status !== "Ready" && !logs.includes(status) && !logs.some(l => l.includes(status))}
            <div class="font-bold mt-1">{status}</div>
          {/if}
        </div>
      </div>
    {/if}
  {/if}
</div>

<FolderSelectorModal 
  bind:isOpen={showFolderSelector}
  rootPath={folderPath}
  onConfirm={handleFolderSelection}
  onCancel={() => showFolderSelector = false}
/>
