<script lang="ts">
  export let progress: number = 0; // 0 to 100
  export let status: string = "";
</script>

<div class="w-full">
  <div class="flex justify-between mb-1">
    <span class="text-sm font-medium text-blue-700">{status}</span>
    <span class="text-sm font-medium text-blue-700">{Math.round(progress)}%</span>
  </div>
  <div class="w-full bg-gray-200 rounded-full h-2.5">
    <div class="bg-blue-600 h-2.5 rounded-full transition-all duration-300" style="width: {progress}%"></div>
  </div>
</div>
