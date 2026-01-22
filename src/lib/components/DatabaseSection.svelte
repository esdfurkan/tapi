<script lang="ts">
  import { api } from '$lib/api_client';
  import { onMount, tick } from 'svelte';
  import { 
    Trash2, Search, Database, RefreshCw, Hash, Edit2, 
    Check, X, Globe, HardDrive, ToggleLeft, CloudUpload, CloudDownload,
    Settings2, ShieldAlert, User, Key
  } from 'lucide-svelte';
  import { fade, slide } from 'svelte/transition';
  import { t } from '$lib/i18n';

  interface HashEntry {
    hash: string;
    name: string;
    created_at: string;
  }

  let entries: HashEntry[] = [];
  let isLoading = false;
  let searchTerm = "";
  let editingHash: string | null = null;
  let editingName = "";

  // Configuration
  let databaseMode = "off"; // off, local, remote
  let remoteUrl = "";
  let remoteToken = "";
  let remoteUser = "";
  let remotePass = "";

  async function loadConfig() {
    try {
      const profile: any = await api.command('load_settings');
      if (profile) {
        databaseMode = profile.database_mode || "off";
        remoteUrl = profile.remote_db_url || "";
        remoteToken = profile.remote_db_token || "";
        remoteUser = profile.remote_db_user || "";
        remotePass = profile.remote_db_pass || "";
      }
    } catch (e) {
      console.error("Failed to load DB config:", e);
    }
  }

  async function updateConfig() {
    try {
      const profile: any = await api.command('load_settings');
      await api.command('save_settings', {
        settings: {
          ...profile,
          database_mode: databaseMode,
          remote_db_url: remoteUrl,
          remote_db_token: remoteToken,
          remote_db_user: remoteUser,
          remote_db_pass: remotePass
        }
      });
      if (databaseMode !== "off") await loadEntries();
    } catch (e) {
      console.error("Failed to update DB config:", e);
    }
  }

  async function loadEntries() {
    if (databaseMode === "off") return;
    isLoading = true;
    try {
      entries = await api.command('list_hash_names');
    } catch (e) {
      console.error("Failed to load database entries:", e);
    } finally {
      isLoading = false;
    }
  }

  async function deleteEntry(hash: string) {
    if (!confirm("Are you sure?")) return;
    try {
      await api.command('delete_hash_entry', { hash });
      await loadEntries();
    } catch (e) {
      console.error("Delete failed:", e);
    }
  }

  async function clearDatabase() {
    if (!confirm("Clear EVERYTHING?")) return;
    try {
      await api.command('clear_all_database');
      await loadEntries();
    } catch (e) {
      console.error("Clear failed:", e);
    }
  }

  async function pullRemote() {
    isLoading = true;
    try {
      await api.command('pull_remote_database');
      await loadEntries();
    } catch (e) {
      alert("Pull failed: " + e);
    } finally {
      isLoading = false;
    }
  }

  async function pushRemote() {
    isLoading = true;
    try {
      await api.command('push_remote_database');
      alert("Changes pushed to remote successfully!");
    } catch (e) {
      alert("Push failed: " + e);
    } finally {
      isLoading = false;
    }
  }

  function startEditing(entry: HashEntry) {
    editingHash = entry.hash;
    editingName = entry.name;
  }

  async function saveEdit() {
    if (!editingHash) return;
    try {
      await api.command('save_hash_name', { hash: editingHash, name: editingName });
      editingHash = null;
      await loadEntries();
    } catch (e) {
      console.error("Save edit failed:", e);
    }
  }

  $: filteredEntries = entries.filter(e => 
    e.hash.toLowerCase().includes(searchTerm.toLowerCase()) || 
    e.name.toLowerCase().includes(searchTerm.toLowerCase())
  );

  onMount(async () => {
    await loadConfig();
    if (databaseMode !== "off") await loadEntries();
  });
</script>

<div class="space-y-6 px-4 md:px-0">
  <!-- Mode Selector Header -->
  <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-3xl p-6 shadow-sm">
      <div class="flex flex-col md:flex-row md:items-center justify-between gap-6">
        <div class="flex items-center gap-4">
            <div class="bg-blue-600 p-3 rounded-2xl text-white shadow-lg shadow-blue-500/20">
                <Database size={24} />
            </div>
            <div>
                <h3 class="text-xl font-bold dark:text-white">Database Mode</h3>
                <p class="text-sm text-gray-500 dark:text-gray-400">Choose how to store mapping data</p>
            </div>
        </div>

        <div class="flex bg-gray-100 dark:bg-gray-800 p-1.5 rounded-2xl gap-1">
            <button 
              on:click={() => { databaseMode = 'off'; updateConfig(); }}
              class="flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-bold transition-all {databaseMode === 'off' ? 'bg-white dark:bg-gray-700 text-gray-900 dark:text-white shadow-sm' : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
            >
                <ToggleLeft size={18} /> Off
            </button>
            <button 
              on:click={() => { databaseMode = 'local'; updateConfig(); }}
              class="flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-bold transition-all {databaseMode === 'local' ? 'bg-white dark:bg-gray-700 text-blue-600 dark:text-blue-400 shadow-sm' : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
            >
                <HardDrive size={18} /> Local
            </button>
            <button 
              on:click={() => { databaseMode = 'remote'; updateConfig(); }}
              class="flex items-center gap-2 px-4 py-2 rounded-xl text-sm font-bold transition-all {databaseMode === 'remote' ? 'bg-white dark:bg-gray-700 text-purple-600 dark:text-purple-400 shadow-sm' : 'text-gray-500 hover:text-gray-700 dark:hover:text-gray-300'}"
            >
                <Globe size={18} /> Remote
            </button>
        </div>
      </div>

      {#if databaseMode === 'remote'}
        <div class="mt-6 pt-6 border-t border-gray-100 dark:border-gray-800" transition:slide>
            <div class="flex items-center justify-between mb-4">
                <h4 class="font-bold dark:text-white flex items-center gap-2">
                    <Settings2 size={18} class="text-purple-500" />
                    Remote Configuration
                </h4>
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div class="space-y-1.5">
                    <label class="text-xs font-bold text-gray-400 uppercase ml-1">Endpoint URL</label>
                    <input 
                      type="text" bind:value={remoteUrl} placeholder="https://your-api.com/sync"
                      class="w-full px-4 py-2.5 bg-gray-50 dark:bg-gray-800 border-none rounded-xl focus:ring-2 focus:ring-purple-500 outline-none dark:text-white"
                      on:blur={updateConfig}
                    />
                </div>
                
                <div class="space-y-1.5">
                    <label class="text-xs font-bold text-gray-400 uppercase ml-1">JWT Token (Optional)</label>
                    <div class="relative">
                        <Key size={14} class="absolute left-3 top-3.5 text-gray-400" />
                        <input 
                          type="password" bind:value={remoteToken} placeholder="Bearer token..."
                          class="w-full pl-9 pr-4 py-2.5 bg-gray-50 dark:bg-gray-800 border-none rounded-xl focus:ring-2 focus:ring-purple-500 outline-none dark:text-white"
                          on:blur={updateConfig}
                        />
                    </div>
                </div>

                <div class="space-y-1.5">
                    <label class="text-xs font-bold text-gray-400 uppercase ml-1">Username (Signin)</label>
                    <div class="relative">
                        <User size={14} class="absolute left-3 top-3.5 text-gray-400" />
                        <input 
                          type="text" bind:value={remoteUser} placeholder="root"
                          class="w-full pl-9 pr-4 py-2.5 bg-gray-50 dark:bg-gray-800 border-none rounded-xl focus:ring-2 focus:ring-purple-500 outline-none dark:text-white"
                          on:blur={updateConfig}
                        />
                    </div>
                </div>

                <div class="space-y-1.5">
                    <label class="text-xs font-bold text-gray-400 uppercase ml-1">Password (Signin)</label>
                    <div class="relative">
                        <Key size={14} class="absolute left-3 top-3.5 text-gray-400" />
                        <input 
                          type="password" bind:value={remotePass} placeholder="••••••••"
                          class="w-full pl-9 pr-4 py-2.5 bg-gray-50 dark:bg-gray-800 border-none rounded-xl focus:ring-2 focus:ring-purple-500 outline-none dark:text-white"
                          on:blur={updateConfig}
                        />
                    </div>
                </div>
            </div>
            <div class="mt-4 p-3 bg-blue-50 dark:bg-blue-900/10 rounded-xl flex items-start gap-3 border border-blue-100 dark:border-blue-900/30">
                <ShieldAlert size={16} class="text-blue-500 mt-0.5" />
                <p class="text-xs text-blue-700 dark:text-blue-300">
                    Use either a <strong>JWT Token</strong> OR <strong>Username/Password</strong>. 
                    Tokens are preferred for precision and security.
                </p>
            </div>
        </div>
      {/if}
  </div>

  {#if databaseMode === 'off'}
    <div class="flex flex-col items-center justify-center py-20 text-center space-y-4" transition:fade>
        <div class="bg-gray-100 dark:bg-gray-800 p-6 rounded-full text-gray-400">
            <ShieldAlert size={48} />
        </div>
        <div class="max-w-sm">
            <h3 class="text-xl font-bold dark:text-white">Database is Disabled</h3>
            <p class="text-gray-500 dark:text-gray-400 mt-2">Enable Local or Remote mode above to start saving and managing your translation mappings.</p>
        </div>
    </div>
  {:else}
    <!-- Database Management Tools -->
    <div class="grid grid-cols-1 md:grid-cols-4 gap-4" transition:fade>
        <div class="md:col-span-2 bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 p-4 rounded-2xl flex items-center gap-4">
            <div class="relative flex-1">
                <Search size={18} class="absolute left-3 top-3 text-gray-400" />
                <input 
                  type="text" bind:value={searchTerm} placeholder="Search mappings..."
                  class="w-full pl-10 pr-4 py-2 bg-gray-50 dark:bg-gray-800 border-none rounded-xl focus:ring-2 focus:ring-blue-500 outline-none dark:text-white text-sm"
                />
            </div>
        </div>

        <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 p-4 rounded-2xl flex items-center justify-between">
            <div class="text-xs font-bold text-gray-400 uppercase">Records</div>
            <div class="text-xl font-black text-blue-500 font-mono">{entries.length}</div>
        </div>

        <div class="flex items-center gap-2">
            {#if databaseMode === 'remote'}
                <button 
                  on:click={pullRemote} 
                  class="flex-1 h-full bg-purple-600 hover:bg-purple-700 text-white rounded-2xl flex flex-col items-center justify-center gap-1 transition-all group"
                  title="Pull from Remote"
                >
                    <CloudDownload size={20} class="group-hover:-translate-y-1 transition-transform" />
                    <span class="text-[10px] font-bold uppercase">Pull</span>
                </button>
                <button 
                  on:click={pushRemote} 
                  class="flex-1 h-full bg-indigo-600 hover:bg-indigo-700 text-white rounded-2xl flex flex-col items-center justify-center gap-1 transition-all group"
                  title="Save to Remote"
                >
                    <CloudUpload size={20} class="group-hover:-translate-y-1 transition-transform" />
                    <span class="text-[10px] font-bold uppercase">Save</span>
                </button>
            {/if}
            <button 
              on:click={loadEntries} 
              class="flex-1 h-full bg-gray-100 dark:bg-gray-800 hover:bg-gray-200 dark:hover:bg-gray-700 text-gray-600 dark:text-gray-300 rounded-2xl flex flex-col items-center justify-center gap-1 transition-all"
            >
                <RefreshCw size={20} class={isLoading ? 'animate-spin' : ''} />
                <span class="text-[10px] font-bold uppercase font-mono">Refresh</span>
            </button>
        </div>
    </div>

    <!-- Main Content Table -->
    <div class="bg-white dark:bg-gray-900 border border-gray-200 dark:border-gray-800 rounded-3xl overflow-hidden shadow-sm">
        <div class="overflow-x-auto">
            <table class="w-full text-left">
                <thead class="bg-gray-50/50 dark:bg-gray-800/50 text-[10px] font-black text-gray-400 uppercase tracking-widest">
                    <tr>
                        <th class="px-8 py-4">Identity / Name</th>
                        <th class="px-8 py-4">System Hash</th>
                        <th class="px-8 py-4 w-20 text-right">Actions</th>
                    </tr>
                </thead>
                <tbody class="divide-y divide-gray-100 dark:divide-gray-800">
                    {#each filteredEntries as entry (entry.hash)}
                        <tr class="group hover:bg-gray-50 dark:hover:bg-gray-800/40 transition-colors">
                            <td class="px-8 py-6">
                                {#if editingHash === entry.hash}
                                    <div class="flex items-center gap-2">
                                        <input 
                                          type="text" bind:value={editingName} 
                                          class="flex-1 bg-white dark:bg-gray-900 border border-blue-500 rounded-lg px-3 py-1.5 text-sm dark:text-white outline-none"
                                          on:keydown={e => e.key === 'Enter' && saveEdit()}
                                        />
                                        <button on:click={saveEdit} class="text-green-500 p-1.5 hover:bg-green-50 dark:hover:bg-green-900/20 rounded-lg"><Check size={18} /></button>
                                        <button on:click={() => editingHash = null} class="text-red-400 p-1.5 hover:bg-red-50 dark:hover:bg-red-900/20 rounded-lg"><X size={18} /></button>
                                    </div>
                                {:else}
                                    <div class="flex items-center gap-3">
                                        <span class="font-bold text-gray-900 dark:text-white">{entry.name}</span>
                                        <button on:click={() => startEditing(entry)} class="opacity-0 group-hover:opacity-100 text-gray-400 hover:text-blue-500 transition-all"><Edit2 size={14} /></button>
                                    </div>
                                {/if}
                            </td>
                            <td class="px-8 py-6">
                                <div class="flex items-center gap-2 text-[10px] font-mono text-gray-400 bg-gray-50 dark:bg-gray-800 border border-gray-200 dark:border-gray-800 px-2 py-1 rounded-lg w-fit">
                                    <Hash size={10} /> {entry.hash}
                                </div>
                            </td>
                            <td class="px-8 py-6 text-right">
                                <button on:click={() => deleteEntry(entry.hash)} class="text-gray-300 hover:text-red-500 transition-colors"><Trash2 size={18} /></button>
                            </td>
                        </tr>
                    {/each}
                </tbody>
            </table>
        </div>
    </div>
  {/if}
</div>
