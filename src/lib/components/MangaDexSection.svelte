<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { downloadDir } from '@tauri-apps/api/path';
  import ProgressBar from './ProgressBar.svelte';
  import MangaLibraryModal from './MangaLibraryModal.svelte';

  export let mangadexUrl: string;
  export let chapterNumber: number;
  export let mangadexFolderName: string;
  export let seriesName: string;
  export let selectedLanguage: string;
  export let useDataSaver: boolean;
  export let useDefaultFolder: boolean;
  export let folderPath: string;
  export let mangadexDownloadPath: string;
  export let isDownloading: boolean;
  export let progress: number;
  export let status: string;
  export let logs: string[];
  export let showLibraryModal: boolean = false;
  export let showInfoModal: boolean = false;
  export let customSelectorRoot: string = "";
  export let isSelectingForMangadex: boolean = false;
  export let showAndroidFolderPicker: boolean = false;

  const languages = [
    { code: "en", name: "English", flag: "🇬🇧" },
    { code: "jp", name: "Japanese", flag: "🇯🇵" },
    { code: "cn", name: "Chinese", flag: "🇨🇳" },
    { code: "kr", name: "Korean", flag: "🇰🇷" },
    { code: "tr", name: "Turkish", flag: "🇹🇷" },
    { code: "fr", name: "French", flag: "🇫🇷" },
    { code: "es", name: "Spanish", flag: "🇪🇸" },
    { code: "ru", name: "Russian", flag: "🇷🇺" },
    { code: "pt", name: "Portuguese", flag: "🇵🇹" },
    { code: "it", name: "Italian", flag: "🇮🇹" },
    { code: "de", name: "German", flag: "🇩🇪" },
    { code: "id", name: "Indonesian", flag: "🇮🇩" },
    { code: "vi", name: "Vietnamese", flag: "🇻🇳" }
  ];

  $: mangadexFolderName = `Chapter_${chapterNumber}`;

  async function selectMangaDexFolder() {
    status = "Opening folder picker...";

    if (navigator.userAgent.toLowerCase().includes("android") || (window as any).__TAURI_MOBILE__) {
      try {
        let base = "/storage/emulated/0/Download"; 
        try {
          base = await downloadDir();
        } catch (err) {}
        
        customSelectorRoot = base;
        isSelectingForMangadex = true;
        showAndroidFolderPicker = true;
        status = "Select a folder...";
        return;
      } catch (e) {
        alert("Error setting Android path: " + e);
        status = "Error: " + e;
        return;
      }
    }

    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: "Select Download Target Directory"
      });
      status = "Ready";
      if (selected) {
        mangadexDownloadPath = selected as string;
      }
    } catch (e) {
      status = "Error: " + e;
      alert("Error selecting folder: " + e);
    }
  }

  async function downloadManga() {
    if (!mangadexUrl) {
      alert("Please enter a MangaDex URL.");
      return;
    }

    let targetDir = useDefaultFolder ? folderPath : mangadexDownloadPath;
    
    if (useDefaultFolder && !folderPath) {
      alert("No default folder selected. Please select a folder in CLI mode first or uncheck 'Use Default Folder'.");
      return;
    }

    if (!useDefaultFolder && !mangadexDownloadPath) {
      alert("Please select a download directory first.");
      return;
    }

    isDownloading = true;
    status = "Downloading...";
    progress = 0;
    logs = [...logs, "Starting download from " + mangadexUrl];

    try {
      let result: string;
      
      if (mangadexUrl.includes("mangadex.org")) {
        result = await invoke('download_mangadex_chapter', {
          url: mangadexUrl,
          folderName: mangadexFolderName,
          targetDir: targetDir,
          useDataSaver: useDataSaver,
          userAgent: navigator.userAgent,
          seriesName: seriesName
        });
      } else {
        result = await invoke('wrapper_download_url', {
          url: mangadexUrl,
          folderName: mangadexFolderName,
          targetDir: targetDir,
          userAgent: navigator.userAgent
        });
      }
      
      await invoke('save_mangadex_history', {
        targetDir: targetDir,
        url: mangadexUrl,
        chapter: parseFloat(chapterNumber.toString()),
        seriesName: seriesName,
        language: selectedLanguage
      });

      logs = [...logs, result];
      status = "Download Complete!";
      chapterNumber = parseFloat((chapterNumber + 1).toFixed(1));
    } catch (e: any) {
      status = "Error: " + e;
      logs = [...logs, "Download Error: " + e];
      alert("Download Failed: " + e);
    } finally {
      isDownloading = false;
    }
  }
</script>

<div class="space-y-4">
  <div class="flex flex-col gap-2">
    <label class="flex flex-col gap-2">
      <span class="text-sm font-medium dark:text-gray-300">Series Name</span>
      <div class="flex gap-2">
        <input 
          type="text" 
          bind:value={seriesName} 
          placeholder="e.g. One Piece"
          class="flex-1 p-2 border rounded bg-gray-50 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
        />
        <button 
          on:click={() => showLibraryModal = true}
          class="px-3 py-2 bg-yellow-100 text-yellow-800 dark:bg-yellow-900/30 dark:text-yellow-200 border border-yellow-200 dark:border-yellow-800 rounded hover:bg-yellow-200 dark:hover:bg-yellow-900/50 flex items-center gap-1"
          title="Open Library"
        >
          📚 Library
        </button>
      </div>
    </label>
  </div>

  <div class="flex flex-col gap-2 relative">
    <label class="flex flex-col gap-2">
      <div class="flex items-center gap-2">
        <span class="text-sm font-medium dark:text-gray-300">URL (MangaDex / Pixiv / nHentai / Any)</span>
        <button 
          on:click={() => showInfoModal = true} 
          class="text-gray-500 hover:text-blue-500 dark:text-gray-400 dark:hover:text-blue-400 rounded-full border border-current w-4 h-4 flex items-center justify-center text-[10px]" 
          title="Examples & Info"
        >
          i
        </button>
      </div>
      <div class="flex gap-2">
        <input 
          type="text" 
          bind:value={mangadexUrl} 
          placeholder="https://mangadex.org/chapter/..."
          class="flex-1 p-2 border rounded bg-gray-50 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
        />
        <button 
          on:click={async () => {
            try {
              const text = await navigator.clipboard.readText();
              if (text) mangadexUrl = text;
            } catch (e) {
              alert("Could not access clipboard. Please paste manually.");
            }
          }}
          class="px-3 py-2 bg-gray-200 dark:bg-gray-600 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
          title="Paste from Clipboard"
        >
          📋
        </button>
      </div>
    </label>
  </div>

  <div class="flex flex-col gap-2">
    <label class="flex flex-col gap-2">
      <span class="text-sm font-medium dark:text-gray-300">Language</span>
      <select bind:value={selectedLanguage} class="p-2 border rounded bg-gray-50 dark:bg-gray-700 dark:border-gray-600 dark:text-white">
        {#each languages as lang}
          <option value={lang.code}>{lang.flag} {lang.name}</option>
        {/each}
      </select>
    </label>
  </div>

  <div class="flex gap-4">
    <div class="w-1/3 flex flex-col gap-2">
      <label class="flex flex-col gap-2">
        <span class="text-sm font-medium dark:text-gray-300">Chapter No</span>
        <div class="flex items-center gap-1">
          <button 
            on:click={() => chapterNumber = Math.max(0, parseFloat((chapterNumber - 1).toFixed(1)))}
            class="w-8 h-10 bg-gray-200 dark:bg-gray-600 rounded flex items-center justify-center hover:bg-gray-300 dark:hover:bg-gray-500 font-bold text-lg"
          >-</button>
          <input 
            type="number" 
            bind:value={chapterNumber} 
            min="0"
            step="0.1"
            class="flex-1 p-2 border rounded bg-gray-50 dark:bg-gray-700 dark:border-gray-600 dark:text-white text-center"
          />
          <button 
            on:click={() => chapterNumber = parseFloat((chapterNumber + 1).toFixed(1))}
            class="w-8 h-10 bg-gray-200 dark:bg-gray-600 rounded flex items-center justify-center hover:bg-gray-300 dark:hover:bg-gray-500 font-bold text-lg"
          >+</button>
        </div>
      </label>
    </div>
    <div class="w-2/3 flex flex-col gap-2">
      <label class="flex flex-col gap-2">
        <span class="text-sm font-medium dark:text-gray-300">Folder Name (Preview)</span>
        <input 
          type="text" 
          bind:value={mangadexFolderName} 
          class="p-2 border rounded bg-gray-50 dark:bg-gray-700 dark:border-gray-600 dark:text-white font-mono"
        />
      </label>
    </div>
  </div>

  <div class="flex items-center gap-2">
    <input type="checkbox" id="useDefaultFolder" bind:checked={useDefaultFolder} class="w-4 h-4" />
    <label for="useDefaultFolder" class="text-sm text-gray-700 dark:text-gray-300">Use default 'Folder Translation' path</label>
  </div>

  <div class="flex items-center gap-2">
    <input type="checkbox" id="useDataSaver" bind:checked={useDataSaver} class="w-4 h-4" />
    <label for="useDataSaver" class="text-sm text-gray-700 dark:text-gray-300">Use Data Saver (Lower Quality)</label>
  </div>
  
  {#if !useDefaultFolder}
    <div class="flex gap-2">
      <input 
        type="text" 
        bind:value={mangadexDownloadPath} 
        placeholder="/path/to/download"
        class="flex-1 p-2 border rounded bg-gray-50 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
        readonly
      />
      <button 
        on:click={selectMangaDexFolder}
        class="bg-gray-200 dark:bg-gray-600 dark:text-white px-4 py-2 rounded hover:bg-gray-300 dark:hover:bg-gray-500"
      >
        Browse
      </button>
    </div>
    <p class="text-xs text-gray-500 dark:text-gray-400">Save to: {mangadexDownloadPath ? mangadexDownloadPath + '/' + mangadexFolderName : 'Select a folder first'}</p>
  {:else if folderPath}
    <p class="text-xs text-gray-500 dark:text-gray-400">Save to: {folderPath}/{mangadexFolderName}</p>
  {:else}
    <p class="text-xs text-red-500">Warning: No default folder selected. Go to 'Folder Translation' to select one, or uncheck the box above.</p>
  {/if}

  {#if isDownloading}
    <ProgressBar {progress} {status} />
  {:else}
    <button 
      on:click={downloadManga}
      class="w-full bg-green-600 text-white py-3 rounded-lg font-medium hover:bg-green-700 transition-colors"
    >
      Download Chapter
    </button>
  {/if}

  {#if logs.length > 0}
    <div class="mt-4 border rounded bg-gray-50 dark:bg-gray-700 dark:border-gray-600 p-3">
      <span class="text-sm font-semibold text-gray-700 dark:text-gray-200 mb-2 block">Logs</span>
      <div class="bg-white dark:bg-gray-800 dark:text-gray-300 border dark:border-gray-600 rounded p-2 h-32 overflow-y-auto text-xs font-mono whitespace-pre-wrap">
        {#each logs as log}
          <div>{log}</div>
        {/each}
      </div>
    </div>
  {/if}
</div>

<MangaLibraryModal 
  show={showLibraryModal} 
  targetDir={useDefaultFolder ? folderPath : mangadexDownloadPath}
  onSelect={(name) => seriesName = name}
  onClose={() => showLibraryModal = false}
/>
