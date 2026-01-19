<script lang="ts">
  import { ChevronDown, Globe, Check } from 'lucide-svelte';
  import { fade, slide } from 'svelte/transition';
  import { createEventDispatcher } from 'svelte';

  export let selectedLanguage = "en";
  
  const dispatch = createEventDispatcher();
  let isOpen = false;
  let isCustom = false;
  let customLangValue = "";

  export let options: { code: string, name: string, flag: string }[] = [];
  
  const defaultLanguages = [
    { code: "en", name: "English", flag: "🇬🇧" },
    { code: "ja", name: "Japanese", flag: "🇯🇵" },
    { code: "ko", name: "Korean", flag: "🇰🇷" },
    { code: "zh", name: "Chinese", flag: "🇨🇳" },
    { code: "es", name: "Spanish", flag: "🇪🇸" },
    { code: "fr", name: "French", flag: "🇫🇷" },
    { code: "de", name: "German", flag: "🇩🇪" },
    { code: "it", name: "Italian", flag: "🇮🇹" },
    { code: "tr", name: "Turkish", flag: "🇹🇷" },
    { code: "ru", name: "Russian", flag: "🇷🇺" },
    { code: "pt", name: "Portuguese", flag: "🇵🇹" },
    { code: "id", name: "Indonesian", flag: "🇮🇩" },
  ];

  $: displayOptions = options.length > 0 ? options : defaultLanguages;

  // Check if initial value is custom
  // Check if initial value is custom
  $: if (selectedLanguage && !displayOptions.some(l => l.code === selectedLanguage) && selectedLanguage !== "custom") {
      isCustom = true;
      customLangValue = selectedLanguage;
  }

  function toggleDropdown() {
    isOpen = !isOpen;
  }

  function selectLanguage(code: string) {
    isCustom = false;
    selectedLanguage = code;
    isOpen = false;
    dispatch('change', { language: code });
  }

  function enableCustom() {
    isCustom = true;
    selectedLanguage = customLangValue || "";
    // Don't close dropdown immediately so user can type
  }

  function updateCustom() {
    selectedLanguage = customLangValue;
    dispatch('change', { language: customLangValue });
  }

  // Click outside to close
  function handleClickOutside(event: MouseEvent) {
    const target = event.target as HTMLElement;
    if (isOpen && !target.closest('.language-selector')) {
      isOpen = false;
    }
  }
</script>

<svelte:window on:click={handleClickOutside} />

<div class="relative language-selector z-50">
  <button 
    class="flex items-center gap-2 px-3 py-2 rounded-lg bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700 transition-all shadow-sm group"
    on:click|stopPropagation={toggleDropdown}
  >
    <div class="p-1 rounded bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400">
      <Globe size={18} />
    </div>
    <span class="font-medium text-sm text-gray-700 dark:text-gray-200">
      {isCustom ? (customLangValue || "Custom") : (displayOptions.find(l => l.code === selectedLanguage)?.name || "Language")}
    </span>
    <ChevronDown size={16} class="text-gray-400 group-hover:text-gray-600 dark:text-gray-500 transition-transform duration-200 {isOpen ? 'rotate-180' : ''}" />
  </button>

  {#if isOpen}
    <div 
      transition:slide={{ duration: 200, axis: 'y' }}
      class="absolute top-full right-0 mt-2 w-56 bg-white dark:bg-gray-800 rounded-xl shadow-xl border border-gray-100 dark:border-gray-700 overflow-hidden"
    >
      <div class="max-h-64 overflow-y-auto py-1">
        {#each displayOptions as lang}
          <button
            class="w-full text-left px-4 py-2 text-sm flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors {selectedLanguage === lang.code ? 'bg-blue-50 dark:bg-blue-900/20 text-blue-600 dark:text-blue-400' : 'text-gray-700 dark:text-gray-300'}"
            on:click={() => selectLanguage(lang.code)}
          >
            <div class="flex items-center gap-3">
              <span class="text-base">{lang.flag}</span>
              <span>{lang.name}</span>
            </div>
            {#if selectedLanguage === lang.code}
              <Check size={14} />
            {/if}
          </button>
        {/each}
        
        <div class="border-t border-gray-100 dark:border-gray-700 mt-1 pt-1">
          <button
            class="w-full text-left px-4 py-2 text-sm flex items-center gap-3 hover:bg-gray-50 dark:hover:bg-gray-700/50 transition-colors {isCustom ? 'text-blue-600 dark:text-blue-400' : 'text-gray-700 dark:text-gray-300'}"
            on:click={enableCustom}
          >
            <span class="text-base">✏️</span>
            <span>Custom Language...</span>
          </button>
          
          {#if isCustom}
            <div class="px-3 pb-2 pt-1" transition:slide>
              <input 
                type="text" 
                placeholder="Target code (e.g. pt-br)" 
                bind:value={customLangValue} 
                on:input={updateCustom}
                class="w-full px-2 py-1.5 text-sm border rounded bg-gray-50 dark:bg-gray-900/50 border-gray-200 dark:border-gray-600 focus:outline-none focus:border-blue-500 dark:text-white"
                on:click|stopPropagation
              />
            </div>
          {/if}
        </div>
      </div>
    </div>
  {/if}
</div>
