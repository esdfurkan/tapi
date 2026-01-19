<script lang="ts">
  import { createEventDispatcher } from 'svelte';
  import { slide } from 'svelte/transition';
  
  export let node: any;
  // Node structure: { name: string, path: string, children?: Node[], selected: boolean, expanded: boolean }

  const dispatch = createEventDispatcher();

  function toggleExpand() {
    if (node.children) {
      node.expanded = !node.expanded;
    }
  }

  function handleCheckboxChange() {
    dispatch('toggle', node);
    if (node.children) {
        updateChildren(node, node.selected);
        node.children = [...node.children];
    }
  }
  
  function updateChildren(parent: any, selected: boolean) {
      if (parent.children) {
          parent.children.forEach((child: any) => {
              child.selected = selected;
              updateChildren(child, selected);
          });
      }
  }
</script>

<div class="select-none h-full">
  <div class="group flex items-center gap-2 py-1.5 px-2 hover:bg-gray-50 dark:hover:bg-white/5 rounded-lg transition-all duration-150 border border-transparent hover:border-gray-100 dark:hover:border-white/10">
    <button 
      on:click={toggleExpand}
      class="w-5 h-5 flex items-center justify-center text-gray-400 hover:text-blue-500 transition-colors focus:outline-none"
    >
      {#if node.children && node.children.length > 0}
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5 transform transition-transform duration-200 {node.expanded ? 'rotate-90' : ''}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="9 18 15 12 9 6"></polyline>
        </svg>
      {:else}
        <div class="w-3.5"></div> 
      {/if}
    </button>
    
    <label class="relative flex items-center cursor-pointer">
      <input 
        type="checkbox" 
        bind:checked={node.selected} 
        on:change={handleCheckboxChange}
        class="peer sr-only"
      />
      <div class="w-4.5 h-4.5 border-2 border-gray-300 dark:border-gray-600 rounded-md bg-white dark:bg-gray-800 peer-checked:bg-blue-600 peer-checked:border-blue-600 transition-all duration-200 flex items-center justify-center">
        <svg xmlns="http://www.w3.org/2000/svg" class="w-3 h-3 text-white scale-0 peer-checked:scale-100 transition-transform duration-200" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="4" stroke-linecap="round" stroke-linejoin="round">
          <polyline points="20 6 9 17 4 12"></polyline>
        </svg>
      </div>
    </label>
    
    <button 
      type="button" 
      class="flex-1 flex items-center gap-2.5 text-sm font-medium text-gray-700 dark:text-gray-300 transition-colors group-hover:text-blue-600 dark:group-hover:text-blue-400 focus:outline-none overflow-hidden" 
      on:click={toggleExpand}
    >
        <span class="flex-shrink-0">
          {#if node.children}
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4.5 h-4.5 text-blue-500/80" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"></path>
              <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"></path>
            </svg>
          {:else}
            <svg xmlns="http://www.w3.org/2000/svg" class="w-4.5 h-4.5 text-gray-400 group-hover:text-blue-400 transition-colors" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <rect x="3" y="3" width="18" height="18" rx="2" ry="2"></rect>
              <line x1="12" y1="8" x2="12" y2="16"></line>
              <line x1="8" y1="12" x2="16" y2="12"></line>
            </svg>
          {/if}
        </span>
        <span class="truncate">{node.name}</span>
    </button>
  </div>

  {#if node.expanded && node.children}
    <div class="ml-4 pl-3 border-l-2 border-gray-100 dark:border-white/5 space-y-0.5" transition:slide={{duration: 200}}>
      {#each node.children as child}
        <svelte:self bind:node={child} on:toggle />
      {/each}
    </div>
  {/if}
</div>

<style>
  .w-4\.5 { width: 1.125rem; }
  .h-4\.5 { height: 1.125rem; }
</style>
