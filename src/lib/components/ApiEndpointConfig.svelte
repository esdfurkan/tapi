<script lang="ts">
  import { t } from '$lib/i18n';
  import KeyValueEditor from './KeyValueEditor.svelte';
  export let storageUrl: string;
  export let ocrUrl: string;
  export let translateUrl: string;
  
  // Storage parameters
  export let storageUrls: string = "";
  export let storageHeaders: string = "";
  
  // OCR parameters
  export let ocrHeaders: string = "";

  // Debugging
  export let saveDebugJson: boolean = false;
  
  // Translate parameters (already in main settings)
  
  let selectedEndpoint: "storage" | "ocr" | "translate" = "translate";
  let showConfig = false;
</script>

<div class="mb-4 border-t pt-4 dark:border-gray-600">
  <button 
    on:click={() => showConfig = !showConfig}
    class="w-full flex items-center justify-between text-sm font-medium dark:text-gray-200 mb-2"
  >
    <span>🔧 Custom API Endpoints & Debug</span>
    <span class="text-xs">{showConfig ? '▼' : '▶'}</span>
  </button>
  
  {#if showConfig}
    <div class="space-y-3 mt-2">
      <!-- Endpoint Selector -->
      <div class="flex gap-2 border-b dark:border-gray-600 pb-2">
        <button
          on:click={() => selectedEndpoint = "storage"}
          class="flex-1 px-3 py-2 text-xs rounded {selectedEndpoint === 'storage' ? 'bg-blue-600 text-white' : 'bg-gray-200 dark:bg-gray-700 dark:text-gray-300'}"
        >
          📦 Storage
        </button>
        <button
          on:click={() => selectedEndpoint = "ocr"}
          class="flex-1 px-3 py-2 text-xs rounded {selectedEndpoint === 'ocr' ? 'bg-blue-600 text-white' : 'bg-gray-200 dark:bg-gray-700 dark:text-gray-300'}"
        >
          👁️ OCR
        </button>
        <button
          on:click={() => selectedEndpoint = "translate"}
          class="flex-1 px-3 py-2 text-xs rounded {selectedEndpoint === 'translate' ? 'bg-blue-600 text-white' : 'bg-gray-200 dark:bg-gray-700 dark:text-gray-300'}"
        >
          🌐 Translate
        </button>
      </div>

      <!-- Storage Config -->
      {#if selectedEndpoint === "storage"}
        <div class="space-y-2 p-3 bg-gray-50 dark:bg-gray-800 rounded">
          <label class="block">
            <span class="text-xs font-medium mb-1 dark:text-gray-300">Storage API URL</span>
            <input 
              type="text" 
              bind:value={storageUrl} 
              placeholder="https://api.toriitranslate.com/api/storage" 
              class="w-full p-2 border rounded text-xs dark:bg-gray-700 dark:border-gray-600 dark:text-white mt-1" 
            />
          </label>
          
          <label class="block">
            <span class="text-xs font-medium mb-1 dark:text-gray-300">Storage URLs (comma-separated)</span>
            <textarea 
              bind:value={storageUrls} 
              placeholder="9abd160f..., e56a56dc..., 73c2095e..."
              rows="3"
              class="w-full p-2 border rounded text-xs dark:bg-gray-700 dark:border-gray-600 dark:text-white mt-1 font-mono"
            ></textarea>
          </label>

          <div class="mt-4">
            <KeyValueEditor 
              bind:jsonString={storageHeaders} 
              title="Storage API Headers"
              description="Add headers like 'Authorization' or 'storage_urls' if required by your private server."
              placeholderKey="Header (e.g. Authorization)"
              placeholderValue="Value (e.g. Bearer xyz...)"
            />
          </div>
          
          <div class="text-xs text-gray-600 dark:text-gray-400 bg-blue-50 dark:bg-blue-900/20 p-2 rounded">
            <strong>Storage API:</strong> Retrieves previously uploaded images by their storage URLs.
            <br/>Headers: <code class="text-[10px]">Authorization, storage_urls</code>
          </div>
        </div>
      {/if}

      <!-- OCR Config -->
      {#if selectedEndpoint === "ocr"}
        <div class="space-y-2 p-3 bg-gray-50 dark:bg-gray-800 rounded">
          <label class="block">
            <span class="text-xs font-medium mb-1 dark:text-gray-300">OCR API URL</span>
            <input 
              type="text" 
              bind:value={ocrUrl} 
              placeholder="https://api.toriitranslate.com/api/ocr" 
              class="w-full p-2 border rounded text-xs dark:bg-gray-700 dark:border-gray-600 dark:text-white mt-1" 
            />
          </label>

          <div class="mt-4">
            <KeyValueEditor 
              bind:jsonString={ocrHeaders} 
              title="OCR API Headers"
              description="Security tokens or authentication headers for the OCR service."
              placeholderKey="Header (e.g. Authorization)"
              placeholderValue="Value (e.g. Bearer token)"
            />
          </div>
          
          <div class="text-xs text-gray-600 dark:text-gray-400 bg-blue-50 dark:bg-blue-900/20 p-2 rounded">
            <strong>OCR API:</strong> Extracts text from images without translation.
            <br/>Headers: <code class="text-[10px]">Authorization</code>
            <br/>Returns: JSON with detected text regions
          </div>
        </div>
      {/if}

      <!-- Translate Config -->
      {#if selectedEndpoint === "translate"}
        <div class="space-y-2 p-3 bg-gray-50 dark:bg-gray-800 rounded">
          <label class="block">
            <span class="text-xs font-medium mb-1 dark:text-gray-300">Translate API URL</span>
            <input 
              type="text" 
              bind:value={translateUrl} 
              placeholder="https://api.toriitranslate.com/api/upload" 
              class="w-full p-2 border rounded text-xs dark:bg-gray-700 dark:border-gray-600 dark:text-white mt-1" 
            />
          </label>
          
          <div class="text-xs text-gray-600 dark:text-gray-400 bg-blue-50 dark:bg-blue-900/20 p-2 rounded">
            <strong>Translate API:</strong> Main translation endpoint.
            <br/>Headers: <code class="text-[10px]">Authorization, target_lang, translator, font, text_align, stroke_disabled, inpaint_only, min_font_size</code>
            <br/>Returns: Translated image (binary)
            <br/><br/>
            <em>Note: Translation parameters (model, font, etc.) are configured in the main settings above.</em>
          </div>
        </div>
      {/if}

      <div class="pt-2 border-t dark:border-gray-700 mt-3 pt-3">
        <label class="flex items-center gap-2">
          <input type="checkbox" bind:checked={saveDebugJson} class="rounded text-blue-600 focus:ring-blue-500 dark:bg-gray-700 dark:border-gray-600">
          <span class="text-xs font-medium dark:text-gray-300">Save API JSON Responses (Debug)</span>
        </label>
        <p class="text-[10px] text-gray-500 dark:text-gray-400 mt-1 ml-6">
          Saves raw JSON responses from OCR/Storage APIs to the output folder.
        </p>
      </div>

      <!-- Reset Button -->
      <button 
        on:click={() => {
          storageUrl = "https://api.toriitranslate.com/api/storage";
          ocrUrl = "https://api.toriitranslate.com/api/ocr";
          translateUrl = "https://api.toriitranslate.com/api/upload";
          storageUrls = "";
        }}
        class="text-xs text-blue-600 dark:text-blue-400 hover:underline"
      >
        🔄 Reset all to defaults
      </button>
    </div>
  {/if}
</div>
