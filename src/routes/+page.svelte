<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";

  let name = $state("");
  let greetMsg = $state("");

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }
</script>

<main class="flex flex-col items-center justify-center pt-[10vh] text-center text-gray-900 dark:text-gray-100">
  <h1 class="text-3xl font-bold">Meeru</h1>

  <div class="flex justify-center my-6">
    <a href="https://vitejs.dev" target="_blank" class="mx-2">
      <img
        src="/vite.svg"
        class="h-24 p-6 transition-all duration-700 hover:drop-shadow-[0_0_2em_#747bff]"
        alt="Vite Logo"
      />
    </a>
    <a href="https://tauri.app" target="_blank" class="mx-2">
      <img
        src="/tauri.svg"
        class="h-24 p-6 transition-all duration-700 hover:drop-shadow-[0_0_2em_#24c8db]"
        alt="Tauri Logo"
      />
    </a>
    <a href="https://kit.svelte.dev" target="_blank" class="mx-2">
      <img
        src="/svelte.svg"
        class="h-24 p-6 transition-all duration-700 hover:drop-shadow-[0_0_2em_#ff3e00]"
        alt="SvelteKit Logo"
      />
    </a>
  </div>

  <p class="text-sm mb-6 text-gray-700 dark:text-gray-300">
    Click on the Tauri, Vite, and SvelteKit logos to learn more.
  </p>

  <form class="flex justify-center gap-2 mb-4" onsubmit={greet}>
    <input
      id="greet-input"
      placeholder="Enter a name..."
      bind:value={name}
      class="rounded-lg border border-transparent px-5 py-2.5 text-base font-medium bg-white dark:bg-gray-900/60 text-gray-900 dark:text-white shadow-sm outline-none transition-colors focus:border-blue-600"
    />
    <button
      type="submit"
      class="rounded-lg border border-transparent px-5 py-2.5 text-base font-medium bg-white dark:bg-gray-900/60 text-gray-900 dark:text-white shadow-sm cursor-pointer outline-none transition-all hover:border-blue-600 active:border-blue-600 active:bg-gray-200 dark:active:bg-gray-900/40"
    >
      Greet
    </button>
  </form>

  {#if greetMsg}
    <p class="mt-2 text-lg font-semibold text-gray-800 dark:text-gray-200">{greetMsg}</p>
  {/if}
</main>
