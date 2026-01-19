<script lang="ts">
  import { Plus, Trash2, Info } from 'lucide-svelte';
  import { createEventDispatcher, onMount, tick } from 'svelte';
  
  export let jsonString: string = "";
  export let placeholderKey: string = "Header Name";
  export let placeholderValue: string = "Value";
  export let title: string = "Custom Headers";
  export let description: string = "Add extra information required by the server.";

  const dispatch = createEventDispatcher();

  let items: { key: string, value: string }[] = [];
  let lastFocusedIndex = -1;
  let keyInputs: HTMLInputElement[] = [];

  // Parse JSON string to items
  $: {
    try {
      if (jsonString && jsonString.trim() !== "") {
        const parsed = JSON.parse(jsonString);
        const newItems = Object.entries(parsed).map(([key, value]) => ({ key, value: String(value) }));
        
        const currentJson = JSON.stringify(items.reduce((acc, curr) => ({...acc, [curr.key]: curr.value}), {}));
        const newJson = JSON.stringify(parsed);
        
        if (currentJson !== newJson) {
             items = newItems;
        }
      } else if (items.length > 0 && jsonString === "") {
           items = [];
      }
    } catch (e) {}
  }

  function updateJson() {
    const obj = items.reduce((acc, curr) => {
      if (curr.key.trim()) {
        acc[curr.key.trim()] = curr.value;
      }
      return acc;
    }, {} as Record<string, string>);
    
    jsonString = JSON.stringify(obj, null, 2);
    dispatch('change', { json: jsonString });
  }

  async function addItem() {
    items = [...items, { key: "", value: "" }];
    lastFocusedIndex = items.length - 1;
    await tick();
    if (keyInputs[lastFocusedIndex]) {
      keyInputs[lastFocusedIndex].focus();
    }
  }

  function removeItem(index: number) {
    items = items.filter((_, i) => i !== index);
    updateJson();
  }
</script>

<div class="bg-gray-50 dark:bg-gray-800/50 rounded-lg p-3 border border-gray-200 dark:border-gray-700 transition-all">
  <div class="flex items-center justify-between mb-2">
    <div class="flex items-center gap-2">
      <span class="text-xs font-semibold text-gray-700 dark:text-gray-200 uppercase tracking-wider">{title}</span>
      <div class="group relative">
        <Info size={14} class="text-gray-400 cursor-help" />
        <div class="absolute left-1/2 -translate-x-1/2 bottom-full mb-2 w-48 p-2 bg-gray-900 text-white text-[10px] rounded shadow-lg opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none z-10">
          {description}
          <div class="absolute left-1/2 -translate-x-1/2 top-full w-0 h-0 border-l-[6px] border-l-transparent border-r-[6px] border-r-transparent border-t-[6px] border-t-gray-900"></div>
        </div>
      </div>
    </div>
  </div>

  {#if items.length === 0}
    <button 
      on:click={addItem}
      class="w-full text-xs text-gray-500 italic mb-2 text-center py-4 bg-white dark:bg-gray-900/30 rounded border border-dashed border-gray-300 dark:border-gray-600 hover:border-blue-400 dark:hover:border-blue-500 hover:bg-blue-50/30 dark:hover:bg-blue-900/10 transition-all group flex flex-col items-center gap-1"
    >
      <Plus size={16} class="text-gray-400 group-hover:text-blue-500 transition-colors" />
      <span>No custom headers set. Click to add one.</span>
    </button>
  {:else}
    <div class="space-y-2 mb-3">
      {#each items as item, i}
        <div class="flex gap-2 items-center group/row animate-in fade-in slide-in-from-top-1 duration-200">
          <input 
            type="text" 
            bind:this={keyInputs[i]}
            bind:value={item.key} 
            on:input={updateJson}
            placeholder={placeholderKey}
            class="flex-1 min-w-0 p-1.5 text-xs border rounded bg-white dark:bg-gray-700 dark:border-gray-600 dark:text-white focus:ring-2 focus:ring-blue-500/30 focus:border-blue-500 outline-none transition-all shadow-sm"
          />
          <span class="text-gray-400 font-bold">:</span>
          <input 
            type="text" 
            bind:value={item.value} 
            on:input={updateJson}
            placeholder={placeholderValue}
            class="flex-[2] min-w-0 p-1.5 text-xs border rounded bg-white dark:bg-gray-700 dark:border-gray-600 dark:text-white focus:ring-2 focus:ring-blue-500/30 focus:border-blue-500 outline-none transition-all shadow-sm"
          />
          <button 
            on:click={() => removeItem(i)}
            class="p-1.5 text-gray-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-900/30 rounded transition-all opacity-0 group-hover/row:opacity-100 focus:opacity-100"
            title="Remove"
          >
            <Trash2 size={14} />
          </button>
        </div>
      {/each}
    </div>
  {/if}

  <button 
    on:click={addItem}
    class="w-full flex items-center justify-center gap-1.5 py-1.5 px-3 text-xs font-medium text-blue-600 dark:text-blue-400 bg-blue-50/50 dark:bg-blue-900/20 border border-blue-200 dark:border-blue-800 rounded-md hover:bg-blue-600 hover:text-white dark:hover:bg-blue-600 dark:hover:text-white transition-all active:scale-95"
  >
    <Plus size={14} />
    Add New Header
  </button>
</div>

