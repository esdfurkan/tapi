<script lang="ts">
  import { api } from '$lib/api_client';
  import { Sun, Moon } from 'lucide-svelte';
  import { t, locale, setLocale } from '$lib/i18n';

  import LanguageSelector from '$lib/components/LanguageSelector.svelte';
  import ApiEndpointConfig from './ApiEndpointConfig.svelte';
  
  export let apiKey: string = "";
  export let model: string = "gemini-2.5-flash";
  export let font: string = "wildwords";
  export let targetLang: string = "en";
  export let isLightTheme: boolean = false;
  export let textAlign: string = "auto";
  export let strokeDisabled: boolean = false;
  export let inpaintOnly: boolean = false;
  export let minFontSize: number = 12;
  
  // API Endpoints
  export let storageUrl: string = "https://api.toriitranslate.com/api/storage";
  export let ocrUrl: string = "https://api.toriitranslate.com/api/ocr";
  export let translateUrl: string = "https://api.toriitranslate.com/api/upload";
  export let storageUrls: string = "";
  export let storageHeaders: string = "";
  export let ocrHeaders: string = "";
  export let saveDebugJson: boolean = false;

  let isShiftHeld = false;
  let keepAdvancedOpen = false;

  function handleKeydown(e: KeyboardEvent) { if (e.key === 'Shift') isShiftHeld = true; }
  function handleKeyup(e: KeyboardEvent) { if (e.key === 'Shift') isShiftHeld = false; }
  
  let selectedModelSelect = "gemini-2.5-flash";
  let customModel = "";
  let selectedFontSelect = "wildwords";
  let customFont = "";

  const models = [
    { value: "default", label: "Default (Gemini 2.5 Flash)" },
    { value: "gemini-3-flash", label: "Gemini 3.0 Flash" },
    { value: "gemini-2.5-flash", label: "Gemini 2.5 Flash" },
    { value: "deepseek", label: "DeepSeek" },
    { value: "grok-4-fast", label: "Grok 4 Fast" },
    { value: "gpt-5", label: "GPT-5" },
    { value: "claude-sonnet-4-5", label: "Claude Sonnet 4.5" },
    { value: "kimi-k2", label: "Kimi K2" },
    { value: "custom", label: "Custom Model..." }
  ];

  const fonts = [
    { value: "wildwords", label: "Wild Words" },
    { value: "badcomic", label: "Bad Comic" },
    { value: "komika", label: "Komika" },
    { value: "bangers", label: "Bangers" },
    { value: "edo", label: "Edo" },
    { value: "ridi", label: "Ridi" },
    { value: "bushidoo", label: "Bushidoo" },
    { value: "hayah", label: "Hayah" },
    { value: "itim", label: "Itim" },
    { value: "mogul", label: "Mogul" },
    { value: "noto", label: "Noto Sans" },
    { value: "custom", label: "Custom Font..." }
  ];

  let lastModel = model;
  let lastFont = font;
  
  $: if (model !== lastModel) {
    lastModel = model;
    if (models.some(m => m.value === model)) { selectedModelSelect = model; } 
    else { selectedModelSelect = "custom"; customModel = model; }
  }

  $: if (font !== lastFont) {
    lastFont = font;
    if (fonts.some(f => f.value === font)) { selectedFontSelect = font; } 
    else { selectedFontSelect = "custom"; customFont = font; }
  }

  function updateModel() {
    if (selectedModelSelect === "custom") { model = customModel; } else { model = selectedModelSelect; }
    lastModel = model;
  }

  function updateFont() {
    if (selectedFontSelect === "custom") { font = customFont; } else { font = selectedFontSelect; }
    lastFont = font;
  }

  async function save() {
    try {
      if (!apiKey || apiKey.trim() === "") { 
        alert("Please enter a valid API Key."); 
        return; 
      }
      
      await api.command('save_settings', { 
        settings: { 
          api_key: apiKey, 
          model, 
          font, 
          language: targetLang, 
          interface_language: $locale,
          theme: isLightTheme ? "light" : "dark",
          text_align: textAlign, 
          stroke_disabled: strokeDisabled, 
          inpaint_only: inpaintOnly, 
          min_font_size: minFontSize,
          total_credits_used: 0,
          storage_url: storageUrl,
          ocr_url: ocrUrl,
          translate_url: translateUrl,
          storage_urls: storageUrls,
          storage_headers: storageHeaders,
          ocr_headers: ocrHeaders,
          save_debug_json: saveDebugJson
        } 
      });
      
      alert("Settings Saved Successfully!");
    } catch (e) { 
      console.error("Save error:", e);
      alert("Error saving settings: " + e); 
    }
  }
</script>

<div class="p-4 border rounded bg-gray-50 dark:bg-gray-800 dark:border-gray-700 transition-colors">
  <h2 class="font-bold mb-4 dark:text-white">{$t('settings.title')}</h2>
  <div class="mb-4">
    <label class="block"><span class="text-sm font-medium mb-1 dark:text-gray-200">{$t('settings.api_key')}</span><input type="password" bind:value={apiKey} class="w-full p-2 border rounded dark:bg-gray-700 dark:border-gray-600 dark:text-white mt-1" /></label>
  </div>
  <div class="mb-4">
    <label class="block"><span class="text-sm font-medium mb-1 dark:text-gray-200">{$t('settings.model')}</span><select bind:value={selectedModelSelect} on:change={updateModel} class="w-full p-2 border rounded mb-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white mt-1">{#each models as m}<option value={m.value}>{m.label}</option>{/each}</select></label>
    {#if selectedModelSelect === 'custom'}<input type="text" bind:value={customModel} on:input={updateModel} class="w-full p-2 border rounded dark:bg-gray-700 dark:border-gray-600 dark:text-white" />{/if}
  </div>

  <div class="mb-4">
    <div class="block"><span class="text-sm font-medium mb-1 dark:text-gray-200">Target Translation Language</span>
      <LanguageSelector 
        bind:selectedLanguage={targetLang} 
        on:change={(e) => targetLang = e.detail.language}
      />
    </div>
    <p class="text-[10px] text-gray-500 mt-1">This is the language the comics will be translated into.</p>
  </div>

  <div class="mb-6 bg-blue-50/50 dark:bg-blue-900/10 p-3 rounded-lg border border-blue-100 dark:border-blue-800">
    <div class="block mb-2">
      <span class="text-xs font-bold text-blue-600 dark:text-blue-400 uppercase tracking-wider">Interface Language (App UI)</span>
    </div>
    <div class="flex items-center justify-between gap-4">
      <div class="flex-1">
        <p class="text-[10px] text-gray-600 dark:text-gray-400 mb-2">You can change the program's language from the header. Click below to manage custom translation files.</p>
        <button 
          on:click={async () => {
            try {
              await api.command('open_translations_folder');
            } catch (e) {
              alert("Failed to open translations folder: " + e);
            }
          }}
          class="text-xs flex items-center gap-1.5 text-blue-600 dark:text-blue-400 hover:underline font-medium"
        >
          <span>📂</span> Open UI Translations Folder
        </button>
      </div>
      <div class="flex items-center gap-2 px-3 py-1.5 bg-white dark:bg-gray-800 rounded border dark:border-gray-700 shadow-sm">
        <span class="text-xs font-mono text-gray-500">Current:</span>
        <span class="text-xs font-bold text-gray-800 dark:text-gray-200 uppercase">{$locale}</span>
      </div>
    </div>
  </div>

  <div class="mb-4">
    <label class="block"><span class="text-sm font-medium mb-1 dark:text-gray-200">{$t('settings.font')}</span><select bind:value={selectedFontSelect} on:change={updateFont} class="w-full p-2 border rounded mb-2 dark:bg-gray-700 dark:border-gray-600 dark:text-white mt-1">{#each fonts as f}<option value={f.value}>{f.label}</option>{/each}</select></label>
  </div>

  
  <ApiEndpointConfig 
    bind:storageUrl
    bind:ocrUrl
    bind:translateUrl
    bind:storageUrls
    bind:storageHeaders
    bind:ocrHeaders
    bind:saveDebugJson
  />
  
  <button on:click={save} class="bg-gray-800 dark:bg-gray-600 text-white px-4 py-2 rounded hover:bg-gray-700 dark:hover:bg-gray-500 w-full transition-colors">{$t('settings.save')}</button>
</div>
<svelte:window on:keydown={handleKeydown} on:keyup={handleKeyup} />