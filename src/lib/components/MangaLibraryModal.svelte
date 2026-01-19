<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';

  export let show = false;
  export let targetDir = "";
  export let onSelect: (seriesName: string) => void;
  export let onClose: () => void;

  let folders: string[] = [];
  let searchQuery = "";
  let isLoading = false;
  let error = "";

  $: if (show && targetDir) {
      loadFolders();
  }

  async function loadFolders() {
      isLoading = true;
      error = "";
      try {
          const result: string[] = await invoke('list_subdirectories', { path: targetDir });
          folders = result;
      } catch (e: any) {
          error = e.toString();
      } finally {
          isLoading = false;
      }
  }

  $: filteredFolders = folders.filter(f => 
       f.toLowerCase().includes(searchQuery.toLowerCase())
  );

  function handleSelect(name: string) {
      onSelect(name);
      onClose();
  }

  function handleBackdrop(e: MouseEvent) {
      if (e.target === e.currentTarget) {
          onClose();
      }
  }
</script>

{#if show}
  <div 
    class="fixed inset-0 bg-black bg-opacity-50 z-50 flex items-center justify-center p-4"
    on:click={handleBackdrop}
    on:keydown={(e) => e.key === 'Escape' && onClose()}
    role="button"
    tabindex="-1"
  >
      <div class="bg-white dark:bg-gray-800 rounded-lg shadow-xl w-full max-w-md p-6 max-h-[80vh] flex flex-col">
          <div class="flex justify-between items-center mb-4">
              <h2 class="text-xl font-bold dark:text-gray-100">Library (Series)</h2>
              <button on:click={onClose} class="text-gray-500 hover:text-gray-700 dark:hover:text-gray-300">✕</button>
          </div>

          <input 
              type="text" 
              bind:value={searchQuery}
              placeholder="Search existing series..."
              class="w-full p-2 mb-4 border rounded bg-gray-50 dark:bg-gray-700 dark:border-gray-600 dark:text-white"
              
          />

          {#if isLoading}
              <div class="text-center py-4 text-gray-500">Loading library...</div>
          {:else if error}
              <div class="text-red-500 text-sm mb-2">Error: {error}</div>
          {:else if filteredFolders.length === 0}
               <div class="text-center py-8 text-gray-400 flex flex-col items-center">
                   <span class="text-3xl mb-2">📂</span>
                   <p>No matching series found.</p>
                   {#if searchQuery}
                    <button 
                        on:click={() => handleSelect(searchQuery)}
                        class="mt-2 text-blue-500 hover:underline"
                    >
                        Create "{searchQuery}"
                    </button>
                   {/if}
               </div>
          {:else}
              <div class="flex-1 overflow-y-auto space-y-1">
                  {#each filteredFolders as folder}
                      <button 
                          on:click={() => handleSelect(folder)}
                          class="w-full text-left px-3 py-2 rounded hover:bg-blue-50 dark:hover:bg-gray-700 dark:text-gray-200 flex items-center gap-2"
                      >
                          <span class="text-yellow-500">📁</span>
                          <span class="truncate">{folder}</span>
                      </button>
                  {/each}
              </div>
          {/if}
          
          <div class="mt-4 text-xs text-gray-500 text-right">
              {filteredFolders.length} series found
          </div>
      </div>
  </div>
{/if}