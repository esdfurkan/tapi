<script lang="ts">
  import { api } from '$lib/api_client';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import { fade } from 'svelte/transition';
  import { FolderOpen, Package, Globe, Settings, X, Info, Sun, Moon } from 'lucide-svelte';
  import { t, locale, setLocale, availableLocales } from '$lib/i18n';
  import ModeCard from '$lib/components/ModeCard.svelte';
  import LanguageSelector from '$lib/components/LanguageSelector.svelte';

  // Map available locales to options format for selector
  $: uiLanguageOptions = availableLocales.map(code => ({
    code,
    name: code === 'en' ? 'English' : (code === 'tr' ? 'Türkçe' : code.toUpperCase()),
    flag: code === 'en' ? '🇬🇧' : (code === 'tr' ? '🇹🇷' : '🏳️')
  }));
  import SettingsPanel from '$lib/components/SettingsPanel.svelte';
  import TranslationSection from '$lib/components/TranslationSection.svelte';
  import MangaDexSection from '$lib/components/MangaDexSection.svelte';
  import FolderSelectorModal from '$lib/components/FolderSelectorModal.svelte';

  let selectedMode = "cli";
  let folderPath = "";
  let status = "Ready";
  let progress = 0;
  let isTranslating = false;
  let logs: string[] = [];
  let showSettingsModal = false;
  let showLibraryModal = false;
  let showInfoModal = false;
  let showFolderSelector = false;
  let showAndroidFolderPicker = false;
  let includedPaths: string[] = [];
  let customSelectorRoot = "";
  let isSelectingForMangadex = false;
  
  let unlistenLog: () => void;
  let unlistenProgress: () => void;
  let unlistenStats: () => void;
  
  // Settings
  let apiKey = "";
  let model = "default";
  let font = "wildwords";
  let targetLang = "en";
  let isLightTheme = false;
  let textAlign = "auto";
  let strokeDisabled = false;
  let inpaintOnly = false;
  let minFontSize = 12;
  
  // API Endpoints
  let storageUrl = "https://api.toriitranslate.com/api/storage";
  let ocrUrl = "https://api.toriitranslate.com/api/ocr";
  let translateUrl = "https://api.toriitranslate.com/api/upload";

  let storageUrls = "";
  let storageHeaders = "";
  let ocrHeaders = "";
  let saveDebugJson = false;

  // MangaDex Settings
  let mangadexUrl = "";
  let chapterNumber = 1;
  let mangadexFolderName = "Chapter_1";
  let seriesName = "";
  let selectedLanguage = "en";
  let useDataSaver = false;
  let useDefaultFolder = true;
  let isDownloading = false;
  let mangadexDownloadPath = "";

  // Stats Overlay
  let showStats = false;
  let systemStats = {
    cpu_usage: 0,
    ram_used: 0,
    ram_total: 0,
    ram_percentage: 0
  };
  let totalCredits = 0;

  $: if (typeof document !== 'undefined') {
    console.log("Applying theme, isLightTheme:", isLightTheme);
    document.documentElement.classList.toggle('dark', !isLightTheme);
    document.documentElement.classList.toggle('light', isLightTheme);
    // Explicitly set style to help tailwind v4
    document.documentElement.style.colorScheme = isLightTheme ? 'light' : 'dark';
  }

  function toggleStats(e: KeyboardEvent) {
    if (e.key === "Shift" && e.location === 2) {
      showStats = !showStats;
    }
  }

  async function saveSettings() {
    try {
      await api.command('save_settings', { 
        settings: { 
          api_key: apiKey, 
          model, 
          font, 
          language: targetLang, 
          interface_language: get(locale),
          theme: isLightTheme ? "light" : "dark",
          text_align: textAlign, 
          stroke_disabled: strokeDisabled, 
          inpaint_only: inpaintOnly, 
          min_font_size: minFontSize,
          total_credits_used: totalCredits,
          storage_url: storageUrl,
          ocr_url: ocrUrl,
          translate_url: translateUrl,
          storage_urls: storageUrls,
          storage_headers: storageHeaders,
          ocr_headers: ocrHeaders,
          save_debug_json: saveDebugJson
        } 
      });
    } catch (e) {
      console.error("Auto-save failed:", e);
    }
  }

  function formatBytes(bytes: number) {
    if (bytes === 0) return '0 B';
    const k = 1024;
    const sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  async function notifyCompletion(title: string, body: string) {
    try {
      if (Math.random() < 0.15) {
        const audio = new Audio('/audio/notification.opus');
        audio.play().catch(e => console.warn("Audio play failed:", e));
      }
    } catch (e) {
      console.warn("Audio init failed:", e);
    }

    try {
      const { isPermissionGranted, requestPermission, sendNotification } = await import('@tauri-apps/plugin-notification');
      let permissionGranted = await isPermissionGranted();
      if (!permissionGranted) {
        const permission = await requestPermission();
        permissionGranted = permission === 'granted';
      }
      if (permissionGranted) {
        sendNotification({ title, body });
      }
    } catch (e) {
      console.error("Notification failed:", e);
    }
  }

  onMount(async () => {
    window.addEventListener('keydown', toggleStats);

    unlistenStats = await listen('system-stats', (event: any) => {
      systemStats = event.payload;
    });

    const creditInterval = setInterval(async () => {
      if (showStats) {
        try {
          const p: any = await api.command('load_settings');
          if (p && p.total_credits_used !== undefined) {
            totalCredits = p.total_credits_used;
          }
        } catch {}
      }
    }, 5000);

    unlistenLog = await listen('translation-log', (event) => {
      if (logs.length > 1000) {
        logs = logs.slice(-900);
      }
      logs = [...logs, event.payload as string];
    });

    unlistenProgress = await listen('translation-progress', (event: any) => {
      const p = event.payload;
      if (p) {
        progress = p.percentage;
        status = p.message;
      }
    });

    try {
      const profile: any = await api.command('load_settings');
      if (profile) {
        if (profile.interface_language) {
             setLocale(profile.interface_language);
        }
        apiKey = profile.api_key || "";
        model = profile.model || "default";
        font = profile.font || "wildwords";
        targetLang = profile.language || "en";
        totalCredits = profile.total_credits_used || 0;
        
        storageUrl = profile.storage_url || "https://api.toriitranslate.com/api/storage";
        ocrUrl = profile.ocr_url || "https://api.toriitranslate.com/api/ocr";
        translateUrl = profile.translate_url || "https://api.toriitranslate.com/api/upload";

        storageUrls = profile.storage_urls || "";
        storageHeaders = profile.storage_headers || "";
        ocrHeaders = profile.ocr_headers || "";
        saveDebugJson = profile.save_debug_json || false;
        
        if (profile.theme === "light") {
          isLightTheme = true;
        } else {
          isLightTheme = false;
        }
      }
    } catch (e) {
      console.error("Failed to load settings", e);
    }
  });

  onDestroy(() => {
    window.removeEventListener('keydown', toggleStats);
    if (unlistenLog) unlistenLog();
    if (unlistenProgress) unlistenProgress();
    if (unlistenStats) unlistenStats();
  });
</script>

<main class="min-h-screen bg-gray-100 dark:bg-gray-900 p-8 transition-colors relative">
  {#if showStats}
    <div class="fixed top-4 right-4 z-50 bg-black/80 text-white p-4 rounded shadow-lg backdrop-blur-sm text-xs font-mono border border-gray-700 w-64 pointer-events-none select-none">
      <h3 class="font-bold text-green-400 mb-2 border-b border-gray-600 pb-1">SYSTEM MONITOR</h3>
      <div class="grid grid-cols-2 gap-y-1">
        <span class="text-gray-400">Total Credits:</span>
        <span class="text-right text-yellow-300">{totalCredits}</span>
        <span class="text-gray-400">CPU Usage:</span>
        <span class="text-right">{systemStats.cpu_usage.toFixed(1)}%</span>
        <span class="text-gray-400">RAM Usage:</span>
        <span class="text-right">{formatBytes(systemStats.ram_used)}</span>
        <span class="text-gray-400">RAM Total:</span>
        <span class="text-right">{formatBytes(systemStats.ram_total)}</span>
        <span class="text-gray-400">RAM Load:</span>
        <span class="text-right">
          <div class="w-full bg-gray-700 h-2 mt-1 rounded-full overflow-hidden">
            <div class="bg-blue-500 h-full" style="width: {systemStats.ram_percentage}%"></div>
          </div>
        </span>
      </div>
      <div class="mt-2 text-gray-500 text-[10px] text-center">Updated every 1.5s</div>
    </div>
  {/if}

  <div class="max-w-4xl mx-auto">
    <header class="mb-8 flex flex-col md:flex-row md:items-center md:justify-between gap-4">
      <div>
        <h1 class="text-3xl font-bold text-gray-900 dark:text-white">{$t('app.title')}</h1>
        <p class="text-gray-600 dark:text-gray-400">{$t('app.subtitle')}</p>
      </div>

      <div class="flex items-center gap-3">
        <!-- Interface Language Selector -->
        <LanguageSelector 
          selectedLanguage={$locale} 
          options={uiLanguageOptions}
          on:change={async (e) => {
            setLocale(e.detail.language);
            await saveSettings();
          }}
        />

        <!-- Theme Toggle -->
        <button 
          class="relative w-14 h-8 rounded-full transition-colors duration-300 focus:outline-none focus:ring-2 focus:ring-blue-500 {isLightTheme ? 'bg-sky-200' : 'bg-indigo-900'}"
          on:click={() => isLightTheme = !isLightTheme}
          aria-label="Toggle Light/Dark Mode"
        >
          <div class="absolute left-1.5 top-1.5 transition-all duration-300 {isLightTheme ? 'opacity-100 scale-100 rotate-0' : 'opacity-0 scale-50 rotate-90'}">
            <Sun size={20} class="text-yellow-600" />
          </div>
          <div class="absolute right-1.5 top-1.5 transition-all duration-300 {isLightTheme ? 'opacity-0 scale-50 -rotate-90' : 'opacity-100 scale-100 rotate-0'}">
            <Moon size={20} class="text-indigo-200" />
          </div>
          <div 
            class="absolute top-1 left-1 w-6 h-6 bg-white rounded-full shadow-md transition-all duration-300 transform {isLightTheme ? 'translate-x-6' : 'translate-x-0'}"
          ></div>
        </button>
      </div>
    </header>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
      <ModeCard 
        title={$t('modes.cli.title')} 
        description={$t('modes.cli.desc')}
        icon={FolderOpen} 
        selected={selectedMode === 'cli'}
        onClick={() => selectedMode = 'cli'}
      />
      <ModeCard 
        title={$t('modes.archive.title')} 
        description={$t('modes.archive.desc')}
        icon={Package} 
        selected={selectedMode === 'archive'}
        onClick={() => selectedMode = 'archive'}
      />
      <ModeCard 
        title={$t('modes.mangadex.title')} 
        description={$t('modes.mangadex.desc')} 
        icon={Globe} 
        selected={selectedMode === 'mangadex'}
        onClick={() => selectedMode = 'mangadex'}
      />
    </div>

    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
      <div class="lg:col-span-2 space-y-6">
          <div class="bg-white dark:bg-gray-800 p-6 rounded-xl shadow-sm border border-gray-100 dark:border-gray-700 transition-all">
            {#key selectedMode}
              <div in:fade={{ duration: 200 }}>
                <h2 class="text-xl font-bold mb-6 dark:text-white flex items-center gap-2">
                  {#if selectedMode === 'cli'}
                    <FolderOpen size={24} class="text-blue-500" />
                    {$t('headers.folder_translation')}
                  {:else if selectedMode === 'archive'}
                    <Package size={24} class="text-blue-500" />
                    {$t('headers.archive_translation')}
                  {:else}
                    <Globe size={24} class="text-blue-500" />
                    {$t('headers.mangadex_downloader')}
                  {/if}
                </h2>
          
          {#if selectedMode === 'cli' || selectedMode === 'archive'}
            <TranslationSection
              bind:selectedMode
              bind:folderPath
              bind:isTranslating
              bind:progress
              bind:status
              bind:logs
              bind:model
              bind:targetLang
              bind:font
              bind:textAlign
              bind:strokeDisabled
              bind:inpaintOnly
              bind:minFontSize
              bind:includedPaths
              bind:showFolderSelector
              bind:customSelectorRoot
              bind:isSelectingForMangadex
              bind:showAndroidFolderPicker
            />
          {:else if selectedMode === 'mangadex'}
            <MangaDexSection
              bind:mangadexUrl
              bind:chapterNumber
              bind:mangadexFolderName
              bind:seriesName
              bind:selectedLanguage
              bind:useDataSaver
              bind:useDefaultFolder
              bind:folderPath
              bind:mangadexDownloadPath
              bind:isDownloading
              bind:progress
              bind:status
              bind:logs
              bind:showLibraryModal
              bind:showInfoModal
              bind:customSelectorRoot
              bind:isSelectingForMangadex
              bind:showAndroidFolderPicker
            />
              {/if}
              </div>
            {/key}
          </div>
      </div>

      <div class="space-y-6 lg:col-span-1">
        <div class="hidden lg:block">
          <SettingsPanel 
            bind:apiKey 
            bind:model 
            bind:font
            bind:targetLang 
            bind:isLightTheme
            bind:textAlign
            bind:strokeDisabled
            bind:inpaintOnly
            bind:minFontSize
            bind:storageUrl
            bind:ocrUrl
            bind:translateUrl

            bind:storageUrls
            bind:storageHeaders
            bind:ocrHeaders
            bind:saveDebugJson
          />
        </div>

        <div class="lg:hidden">
          <button 
            on:click={() => showSettingsModal = true}
            class="w-full bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-200 py-3 rounded-lg font-medium flex items-center justify-center gap-2 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors shadow-sm"
          >
            <Settings size={20} /> {$t('app.settings')}
          </button>
        </div>
      </div>
    </div>
  </div>

  {#if showSettingsModal}
    <div class="fixed inset-0 bg-black/50 backdrop-blur-sm z-[60] flex items-center justify-center p-4">
      <div class="bg-white dark:bg-gray-900 rounded-xl shadow-2xl w-full max-w-md max-h-[90vh] overflow-y-auto relative">
          <button 
            on:click={() => showSettingsModal = false}
            class="absolute top-4 right-4 text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors p-2"
          >
            <X size={20} />
          </button>
        <div class="p-2">
          <SettingsPanel 
            bind:apiKey 
            bind:model 
            bind:font
            bind:targetLang 
            bind:isLightTheme
            bind:textAlign
            bind:strokeDisabled
            bind:inpaintOnly
            bind:minFontSize
            bind:storageUrl
            bind:ocrUrl
            bind:translateUrl

            bind:storageUrls
            bind:storageHeaders
            bind:ocrHeaders
            bind:saveDebugJson
          />
        </div>
        <div class="p-4 border-t border-gray-200 dark:border-gray-700 mt-2">
          <button 
            on:click={() => showSettingsModal = false}
            class="w-full bg-blue-600 text-white py-2 rounded-lg font-medium"
          >
            Done
          </button>
        </div>
      </div>
    </div>
  {/if}

  {#if showAndroidFolderPicker}
    <FolderSelectorModal 
      rootPath={customSelectorRoot} 
      isOpen={showAndroidFolderPicker}
      onConfirm={(paths) => {
        if (paths.length > 0) {
          const chosen = paths[0];
          if (isSelectingForMangadex) {
            mangadexDownloadPath = chosen;
          } else {
            folderPath = chosen;
            includedPaths = [];
          }
        } else {
          if (isSelectingForMangadex) {
            mangadexDownloadPath = customSelectorRoot;
          } else {
            folderPath = customSelectorRoot;
          }
        }
        showAndroidFolderPicker = false;
        status = "Ready";
      }}
      onCancel={() => {
        showAndroidFolderPicker = false;
        status = "Cancelled selection";
      }}
    />
  {/if}

  {#if showInfoModal}
    <div class="fixed inset-0 bg-black/50 z-50 flex items-center justify-center backdrop-blur-sm p-4">
      <div class="bg-white dark:bg-gray-800 rounded-xl shadow-2xl max-w-lg w-full overflow-hidden border border-gray-100 dark:border-gray-700">
        <div class="px-6 py-4 border-b border-gray-100 dark:border-gray-700 flex justify-between items-center bg-gray-50 dark:bg-gray-800/50">
          <h3 class="font-bold text-lg dark:text-white flex items-center gap-2">
            <Info size={20} class="text-blue-500" />
            Supported Sites & Disclaimer
          </h3>
          <button on:click={() => showInfoModal = false} class="text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 transition-colors">
            <X size={20} />
          </button>
        </div>
        
        <div class="p-6 space-y-4 max-h-[70vh] overflow-y-auto">
          <div class="bg-red-50 dark:bg-red-900/20 border border-red-100 dark:border-red-800 rounded-lg p-3 text-sm text-red-800 dark:text-red-200">
            <strong>Disclaimer:</strong> This software is provided for educational and personal archiving purposes only. 
            The developers assume no liability for the content downloaded or how it is used. 
            Please respect copyright laws and the terms of service of the respective websites.
          </div>

          <div class="space-y-2">
            <h4 class="font-semibold dark:text-white">Supported Platforms</h4>
            <ul class="list-disc list-inside text-sm space-y-1 text-gray-700 dark:text-gray-300">
              <li><span class="font-medium text-orange-600">MangaDex</span>: Full Chapter & Data Saver support.</li>
              <li><span class="font-medium text-blue-500">Pixiv</span>: Auto-fetches "Original" quality artworks.</li>
              <li><span class="font-medium text-pink-500">nHentai</span>: Auto-resolves Gallery IDs.</li>
              <li><span class="font-medium text-green-600">Generic</span>: Scrapes images from ANY webpage.</li>
            </ul>
          </div>
        </div>

        <div class="p-4 border-t border-gray-100 dark:border-gray-700 bg-gray-50 dark:bg-gray-800/50 text-right">
          <button on:click={() => showInfoModal = false} class="bg-gray-800 text-white px-4 py-2 rounded-lg text-sm hover:bg-gray-700">Understood</button>
        </div>
      </div>
    </div>
  {/if}
</main>
