<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { fade, scale } from 'svelte/transition';
  import FolderTreeItem from './FolderTreeItem.svelte';

  export let rootPath: string = "";
  export let isOpen: boolean = false;
  export let onConfirm: (selectedPaths: string[]) => void;
  export let onCancel: () => void;

  let rootNode: any = null;
  let isLoading = false;
  let searchQuery = "";

  $: if (isOpen && rootPath) {
      loadTree(rootPath);
  }

  async function loadTree(path: string) {
    isLoading = true;
    try {
        const structure = await invoke('get_directory_structure', { path });
        rootNode = structure;
        if (rootNode) { 
            rootNode.expanded = true; 
            rootNode.selected = true;
            selectAll(rootNode, true);
        }
    } catch (e) {
        console.error("Failed to load tree", e);
    } finally {
        isLoading = false;
    }
  }

  function selectAll(node: any, val: boolean) {
      node.selected = val;
      if (node.children) {
          node.children.forEach((c: any) => selectAll(c, val));
      }
  }

  function getSelectedNodes(node: any): string[] {
      let paths: string[] = [];
      if (node.selected) {
          paths.push(node.path);
      }
      if (node.children) {
          node.children.forEach((c: any) => {
              paths = [...paths, ...getSelectedNodes(c)];
          });
      }
      return paths;
  }

  function handleConfirm() {
      if (rootNode) {
          onConfirm(getSelectedNodes(rootNode));
      }
      isOpen = false;
  }
  
  function goUp() {
      if (!rootPath) return;
      const parts = rootPath.split(/[/\\]/);
      if (parts.length > 1) {
          parts.pop();
          let newPath = parts.join("/");
          if (newPath === "" && rootPath.startsWith("/")) newPath = "/";
          if (newPath.length < rootPath.length) {
                rootPath = newPath; 
          }
      }
  }

  // Reactive selection count
  $: selectedCount = rootNode ? getSelectedNodes(rootNode).filter(p => !rootNode.children?.some((c:any) => p === c.path)).length : 0;
  // Note: Simple count for UI feedback. Accurate count would distinguish files vs folders.

</script>

{#if isOpen}
  <div class="fixed inset-0 bg-black/60 backdrop-blur-md z-50 flex items-center justify-center p-4 sm:p-8" transition:fade={{duration: 200}}>
    <div 
      class="bg-white dark:bg-gray-900 rounded-3xl shadow-2xl w-full max-w-2xl max-h-[85vh] flex flex-col border border-gray-200 dark:border-white/10 overflow-hidden"
      transition:scale={{duration: 250, start: 0.95}}
    >
      <!-- Header -->
      <div class="p-6 border-b border-gray-100 dark:border-white/5 bg-gray-50/50 dark:bg-white/5">
        <div class="flex justify-between items-center mb-4">
          <div>
            <h3 class="text-xl font-bold text-gray-900 dark:text-white flex items-center gap-2">
              <svg xmlns="http://www.w3.org/2000/svg" class="w-6 h-6 text-blue-500" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                <path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"></path>
              </svg>
              Select Content
            </h3>
            <p class="text-xs text-gray-500 dark:text-gray-400 mt-1 font-medium truncate max-w-sm" title={rootPath}>
              {rootPath}
            </p>
          </div>
          <button on:click={onCancel} class="w-8 h-8 flex items-center justify-center rounded-full hover:bg-gray-200 dark:hover:bg-white/10 text-gray-400 dark:text-gray-500 transition-colors" aria-label="Close">
            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
          </button>
        </div>

        <div class="flex gap-2">
          <div class="relative flex-1 group">
            <svg xmlns="http://www.w3.org/2000/svg" class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400 group-focus-within:text-blue-500 transition-colors" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
            <input 
              type="text" 
              bind:value={searchQuery}
              placeholder="Filter items..."
              class="w-full pl-10 pr-4 py-2.5 bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 rounded-2xl text-sm dark:text-white focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 transition-all outline-none"
            />
          </div>
          <button 
            on:click={goUp} 
            class="p-2.5 px-4 bg-white dark:bg-gray-800 border border-gray-200 dark:border-white/10 rounded-2xl hover:bg-gray-50 dark:hover:bg-white/5 transition-all flex items-center gap-2 group shadow-sm active:scale-95"
            title="Go up one level"
          >
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-gray-400 group-hover:text-blue-500 group-hover:-translate-y-0.5 transition-all" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 15l-6-6-6 6"></path></svg>
            <span class="text-xs font-bold text-gray-500 dark:text-gray-400 group-hover:text-gray-700 dark:group-hover:text-gray-300">UP</span>
          </button>
        </div>
      </div>
      
      <!-- Content -->
      <div class="flex-1 overflow-y-auto p-6 space-y-2 min-h-[300px] custom-scrollbar">
         {#if isLoading}
            <div class="flex flex-col items-center justify-center h-full space-y-4 py-20" in:fade>
                <div class="relative w-12 h-12">
                  <div class="absolute inset-0 border-4 border-blue-500/20 rounded-full"></div>
                  <div class="absolute inset-0 border-4 border-t-blue-500 rounded-full animate-spin"></div>
                </div>
                <span class="text-sm font-semibold text-gray-500 dark:text-gray-400 tracking-wide uppercase">Searching files...</span>
            </div>
         {:else if rootNode}
            <div class="animate-in fade-in slide-in-from-bottom-2 duration-300">
              <FolderTreeItem bind:node={rootNode} />
            </div>
         {:else}
            <div class="flex flex-col items-center justify-center h-full text-gray-500 py-20" in:fade>
              <svg xmlns="http://www.w3.org/2000/svg" class="w-16 h-16 opacity-10 mb-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"></circle><line x1="21" y1="21" x2="16.65" y2="16.65"></line></svg>
              <p class="font-medium tracking-tight">Access denied or empty folder.</p>
            </div>
         {/if}
      </div>

      <!-- Footer -->
      <div class="p-6 border-t border-gray-100 dark:border-white/5 flex flex-col sm:flex-row justify-between items-center gap-4 bg-gray-50/50 dark:bg-white/5">
         <div class="flex items-center gap-3">
            <div class="flex -space-x-2">
              <div class="w-8 h-8 rounded-full bg-blue-100 dark:bg-blue-900/30 flex items-center justify-center border-2 border-white dark:border-gray-900 z-10">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4 text-blue-600 dark:text-blue-400" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 11.08V12a10 10 0 1 1-5.93-9.14"></path><polyline points="22 4 12 14.01 9 11.01"></polyline></svg>
              </div>
            </div>
            <div>
              <span class="text-xs font-bold text-gray-400 uppercase tracking-tighter">Ready to translate</span>
              <p class="text-sm font-extrabold text-gray-700 dark:text-gray-200">~{selectedCount} selected items</p>
            </div>
         </div>

         <div class="flex gap-3 w-full sm:w-auto">
           <button 
             on:click={onCancel}
             class="flex-1 sm:flex-none px-6 py-2.5 text-sm font-bold text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 transition-colors uppercase tracking-widest"
           >
             Cancel
           </button>
           <button 
             on:click={handleConfirm}
             class="flex-1 sm:flex-none px-8 py-3 bg-blue-600 text-white font-bold rounded-2xl shadow-lg shadow-blue-500/20 hover:bg-blue-700 hover:shadow-blue-500/40 hover:-translate-y-0.5 active:scale-95 transition-all uppercase tracking-widest text-xs"
           >
             Confirm Selection
           </button>
         </div>
      </div>
    </div>
  </div>
{/if}

<style>
  .custom-scrollbar::-webkit-scrollbar {
    width: 6px;
  }
  .custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
  }
  .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(156, 163, 175, 0.2);
    border-radius: 10px;
  }
  :global(.dark) .custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
  }
</style>
