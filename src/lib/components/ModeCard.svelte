<script lang="ts">
  export let title: string;
  export let description: string;
  export let icon: any; // Lucide component
  export let selected: boolean = false;
  export let onClick: () => void;
</script>

<button 
  class="relative p-6 border rounded-xl text-left transition-all duration-200 hover:shadow-lg group
  {selected 
    ? 'border-blue-500 bg-blue-50/50 dark:bg-blue-900/20 ring-2 ring-blue-500/20 shadow-blue-500/10' 
    : 'border-gray-200 bg-white dark:bg-gray-800 dark:border-gray-700 hover:border-gray-300 dark:hover:border-gray-600'}"
  on:click={onClick}
>
  <div class="flex items-center gap-3 mb-3">
    <div class="p-2 rounded-lg transition-colors {selected ? 'bg-blue-100 dark:bg-blue-800/50 text-blue-600 dark:text-blue-400' : 'bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 group-hover:text-blue-500 dark:group-hover:text-blue-400'}">
      <svelte:component this={icon} size={24} />
    </div>
    <h3 class="font-bold text-lg text-gray-900 dark:text-gray-100">{title}</h3>
  </div>
  <p class="text-gray-600 text-sm dark:text-gray-400 leading-relaxed">{description}</p>
</button>
