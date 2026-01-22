<script lang="ts">
  import { api } from '$lib/api_client';
  import { listen } from '@tauri-apps/api/event';
  import { onMount, onDestroy } from 'svelte';
  import { fade } from 'svelte/transition';
  import { FolderOpen, Package, Globe, Settings, X, Info, Sun, Moon, Database, LayoutDashboard, ChevronLeft, ChevronRight } from 'lucide-svelte';
  import { t, locale, setLocale, availableLocales } from '$lib/i18n';
  import LanguageSelector from '$lib/components/LanguageSelector.svelte';
  import SettingsPanel from '$lib/components/SettingsPanel.svelte';
  import TranslationSection from '$lib/components/TranslationSection.svelte';
  import MangaDexSection from '$lib/components/MangaDexSection.svelte';
  import DatabaseSection from '$lib/components/DatabaseSection.svelte';
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
  let isSidebarCollapsed = false;
  
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
    document.documentElement.classList.toggle('dark', !isLightTheme);
    document.documentElement.classList.toggle('light', isLightTheme);
    document.documentElement.style.colorScheme = isLightTheme ? 'light' : 'dark';
  }

  $: uiLanguageOptions = availableLocales.map(code => ({
    code,
    name: code === 'en' ? 'English' : (code === 'tr' ? 'Türkçe' : code.toUpperCase()),
    flag: code === 'en' ? '🇬🇧' : (code === 'tr' ? '🇹🇷' : '🏳️')
  }));

  async function saveSettings() {
    try {
      await api.command('save_settings', { 
        settings: { 
          api_key: apiKey, model, font, language: targetLang, 
          interface_language: locale.get(),
          theme: isLightTheme ? "light" : "dark",
          text_align: textAlign, stroke_disabled: strokeDisabled, 
          inpaint_only: inpaintOnly, min_font_size: minFontSize,
          total_credits_used: totalCredits,
          storage_url: storageUrl, ocr_url: ocrUrl, translate_url: translateUrl,
          storage_urls: storageUrls, storage_headers: storageHeaders, 
          ocr_headers: ocrHeaders, save_debug_json: saveDebugJson
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

  onMount(async () => {
    unlistenStats = await listen('system-stats', (event: any) => {
      systemStats = event.payload;
    });

    unlistenLog = await listen('translation-log', (event) => {
      if (logs.length > 1000) logs = logs.slice(-900);
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
        if (profile.interface_language) setLocale(profile.interface_language);
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
        isLightTheme = profile.theme === "light";
      }
    } catch (e) {
      console.error("Failed to load settings", e);
    }
  });

  onDestroy(() => {
    if (unlistenLog) unlistenLog();
    if (unlistenProgress) unlistenProgress();
    if (unlistenStats) unlistenStats();
  });

  const sidebarItems = [
    { id: 'cli', icon: FolderOpen, label: 'modes.cli.title' },
    { id: 'archive', icon: Package, label: 'modes.archive.title' },
    { id: 'mangadex', icon: Globe, label: 'modes.mangadex.title' },
    { id: 'database', icon: Database, label: 'database.title' },
  ];
</script>

<div class="flex h-screen bg-gray-50 dark:bg-gray-950 transition-colors overflow-hidden">
  <!-- Sidebar -->
  <aside 
    class="bg-white dark:bg-gray-900 border-r border-gray-200 dark:border-gray-800 flex flex-col transition-all duration-300 z-50 {isSidebarCollapsed ? 'w-20' : 'w-64'}"
  >
    <div class="p-6 flex items-center gap-3 overflow-hidden whitespace-nowrap">
      <div class="bg-blue-600 p-2 rounded-lg text-white flex-shrink-0 shadow-lg shadow-blue-500/20">
        <LayoutDashboard size={24} />
      </div>
      <span class="font-bold text-xl dark:text-white transition-opacity duration-300 {isSidebarCollapsed ? 'opacity-0' : 'opacity-100'}">TAPI</span>
    </div>

    <nav class="flex-1 px-4 space-y-2 mt-4">
      {#each sidebarItems as item}
        <button 
          on:click={() => selectedMode = item.id}
          class="w-full flex items-center gap-4 px-3 py-3 rounded-xl transition-all group relative {selectedMode === item.id ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400' : 'text-gray-500 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-800'}"
        >
          <item.icon size={22} class="flex-shrink-0" />
          <span class="font-medium whitespace-nowrap transition-opacity duration-300 {isSidebarCollapsed ? 'opacity-0 pointer-events-none' : 'opacity-100'}">{$t(item.label)}</span>
          
          {#if isSidebarCollapsed}
            <div class="absolute left-16 bg-gray-900 text-white px-2 py-1 rounded text-xs opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity ml-2 z-50 whitespace-nowrap shadow-xl">
              {$t(item.label)}
            </div>
          {/if}
        </button>
      {/each}
    </nav>

    <div class="p-4 border-t border-gray-200 dark:border-gray-800 space-y-4">
      <!-- Collapse Toggle -->
      <button 
        on:click={() => isSidebarCollapsed = !isSidebarCollapsed}
        class="w-full flex items-center gap-4 px-3 py-2 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors"
      >
        {#if isSidebarCollapsed}
          <ChevronRight size={20} class="mx-auto" />
        {:else}
          <ChevronLeft size={20} />
          <span class="text-sm">Collapse Sidebar</span>
        {/if}
      </button>

      <button 
        on:click={() => showSettingsModal = true}
        class="w-full flex items-center gap-4 px-3 py-3 text-gray-500 hover:bg-gray-100 dark:hover:bg-gray-800 rounded-lg transition-colors group relative"
      >
        <Settings size={22} />
        <span class="font-medium whitespace-nowrap transition-opacity {isSidebarCollapsed ? 'opacity-0' : 'opacity-100'}">{$t('app.settings')}</span>
        {#if isSidebarCollapsed}
            <div class="absolute left-16 bg-gray-900 text-white px-2 py-1 rounded text-xs opacity-0 group-hover:opacity-100 pointer-events-none transition-opacity ml-2 z-50 shadow-xl">
              {$t('app.settings')}
            </div>
        {/if}
      </button>
    </div>
  </aside>

  <!-- Main Content -->
  <main class="flex-1 flex flex-col min-w-0 overflow-hidden relative">
    <!-- Header -->
    <header class="h-16 bg-white dark:bg-gray-900 border-b border-gray-200 dark:border-gray-800 flex items-center justify-between px-8 z-40">
      <div class="flex items-center gap-4">
        <h2 class="text-lg font-bold text-gray-900 dark:text-white uppercase tracking-wider">
          {#if selectedMode === 'cli'}{$t('headers.folder_translation')}
          {:else if selectedMode === 'archive'}{$t('headers.archive_translation')}
          {:else if selectedMode === 'mangadex'}{$t('headers.mangadex_downloader')}
          {:else if selectedMode === 'database'}{$t('headers.database_management')}
          {/if}
        </h2>
      </div>

      <div class="flex items-center gap-6">
        <!-- Stats summary (always visible header) -->
        <div class="hidden md:flex items-center gap-4 text-xs font-mono text-gray-500">
           <div class="flex items-center gap-2">
             <div class="w-1.5 h-1.5 rounded-full bg-green-500 animate-pulse"></div>
             CPU: {systemStats.cpu_usage.toFixed(0)}%
           </div>
           <div>RAM: {systemStats.ram_percentage.toFixed(0)}%</div>
        </div>

        <LanguageSelector 
          selectedLanguage={$locale} 
          options={uiLanguageOptions}
          on:change={async (e) => {
            setLocale(e.detail.language);
            await saveSettings();
          }}
        />

        <button 
          class="p-2 text-gray-500 hover:text-gray-900 dark:hover:text-white rounded-lg transition-colors"
          on:click={() => isLightTheme = !isLightTheme}
        >
          {#if isLightTheme}<Moon size={22} />{:else}<Sun size={22} />{/if}
        </button>
      </div>
    </header>

    <!-- Scrollable Body -->
    <div class="flex-1 overflow-y-auto p-8 relative">
      <div class="max-w-6xl mx-auto h-full">
        {#key selectedMode}
          <div in:fade={{ duration: 200 }} class="h-full">
            {#if selectedMode === 'cli' || selectedMode === 'archive'}
              <div class="grid grid-cols-1 lg:grid-cols-3 gap-8 h-full">
                <div class="lg:col-span-2">
                  <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-2xl p-8 shadow-sm">
                    <TranslationSection
                      bind:selectedMode bind:folderPath bind:isTranslating
                      bind:progress bind:status bind:logs bind:model
                      bind:targetLang bind:font bind:textAlign
                      bind:strokeDisabled bind:inpaintOnly bind:minFontSize
                      bind:includedPaths bind:showFolderSelector
                      bind:customSelectorRoot bind:isSelectingForMangadex
                      bind:showAndroidFolderPicker
                    />
                  </div>
                </div>
                <div class="hidden lg:block">
                  <SettingsPanel 
                    bind:apiKey bind:model bind:font bind:targetLang 
                    bind:isLightTheme bind:textAlign bind:strokeDisabled
                    bind:inpaintOnly bind:minFontSize bind:storageUrl
                    bind:ocrUrl bind:translateUrl bind:storageUrls
                    bind:storageHeaders bind:ocrHeaders bind:saveDebugJson
                  />
                </div>
              </div>
            {:else if selectedMode === 'mangadex'}
              <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-2xl p-8 shadow-sm">
                <MangaDexSection
                  bind:mangadexUrl bind:chapterNumber bind:mangadexFolderName
                  bind:seriesName bind:selectedLanguage bind:useDataSaver
                  bind:useDefaultFolder bind:folderPath bind:mangadexDownloadPath
                  bind:isDownloading bind:progress bind:status bind:logs
                  bind:showLibraryModal bind:showInfoModal bind:customSelectorRoot
                  bind:isSelectingForMangadex bind:showAndroidFolderPicker
                />
              </div>
            {:else if selectedMode === 'database'}
              <DatabaseSection />
            {/if}
          </div>
        {/key}
      </div>
    </div>
  </main>

  <!-- Settings Modal (Mobile & Generic) -->
  {#if showSettingsModal}
    <div class="fixed inset-0 bg-black/60 backdrop-blur-md z-[100] flex items-center justify-center p-4" transition:fade>
      <div class="bg-white dark:bg-gray-950 rounded-2xl shadow-2xl w-full max-w-2xl max-h-[90vh] overflow-hidden flex flex-col">
        <div class="p-6 border-b border-gray-200 dark:border-gray-800 flex justify-between items-center bg-gray-50/50 dark:bg-gray-900/50">
          <h3 class="text-xl font-bold dark:text-white flex items-center gap-3">
            <Settings size={24} class="text-blue-500" />
            {$t('settings.title')}
          </h3>
          <button on:click={() => showSettingsModal = false} class="p-2 hover:bg-gray-200 dark:hover:bg-gray-800 rounded-lg transition-colors text-gray-400">
            <X size={24} />
          </button>
        </div>
        <div class="flex-1 overflow-y-auto p-6">
          <SettingsPanel 
            bind:apiKey bind:model bind:font bind:targetLang 
            bind:isLightTheme bind:textAlign bind:strokeDisabled
            bind:inpaintOnly bind:minFontSize bind:storageUrl
            bind:ocrUrl bind:translateUrl bind:storageUrls
            bind:storageHeaders bind:ocrHeaders bind:saveDebugJson
          />
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
          if (isSelectingForMangadex) mangadexDownloadPath = chosen;
          else { folderPath = chosen; includedPaths = []; }
        }
        showAndroidFolderPicker = false;
        status = "Ready";
      }}
      onCancel={() => { showAndroidFolderPicker = false; status = "Cancelled selection"; }}
    />
  {/if}
</div>

<style>
  :global(body) {
    overflow: hidden;
  }
</style>
