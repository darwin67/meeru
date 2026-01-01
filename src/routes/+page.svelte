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

  <!-- Google OAuth2 Section -->
  <div class="mb-8 p-6 bg-white dark:bg-gray-900/60 rounded-lg shadow-lg max-w-md w-full">
    <h2 class="text-xl font-semibold mb-4">Google OAuth2 Authentication</h2>

    {#if !userInfo}
      <button
        onclick={handleGoogleSignIn}
        disabled={isAuthenticating}
        class="w-full rounded-lg border border-transparent px-5 py-3 text-base font-medium bg-blue-600 text-white shadow-sm cursor-pointer outline-none transition-all hover:bg-blue-700 active:bg-blue-800 disabled:opacity-50 disabled:cursor-not-allowed"
      >
        {isAuthenticating ? "Signing in..." : "Sign in with Google"}
      </button>
    {:else}
      <div class="mb-4 p-4 bg-green-100 dark:bg-green-900/30 rounded-lg">
        <p class="text-green-800 dark:text-green-200 font-semibold mb-2">Authenticated!</p>
        <p class="text-sm text-gray-700 dark:text-gray-300">
          Token expires: {userInfo.expiresAt.toLocaleString()}
        </p>
      </div>
      <button
        onclick={handleGoogleSignOut}
        class="w-full rounded-lg border border-transparent px-5 py-3 text-base font-medium bg-red-600 text-white shadow-sm cursor-pointer outline-none transition-all hover:bg-red-700 active:bg-red-800"
      >
        Sign Out
      </button>
    {/if}

    {#if authStatus}
      <p class="mt-4 text-sm {authStatus.includes('failed') ? 'text-red-600' : 'text-gray-700 dark:text-gray-300'}">
        {authStatus}
      </p>
    {/if}
  </div>

  <!-- Greet Demo Section -->
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

<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { signIn, signOut } from "@choochmeque/tauri-plugin-google-auth-api";

  let name = $state("");
  let greetMsg = $state("");
  let authStatus = $state("");
  let userInfo = $state<any>(null);
  let isAuthenticating = $state(false);

  async function greet(event: Event) {
    event.preventDefault();
    // Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
    greetMsg = await invoke("greet", { name });
  }

  async function handleGoogleSignIn() {
    isAuthenticating = true;
    authStatus = "Signing in...";

    try {
      // Get OAuth config from Rust
      const config = await invoke<{
        client_id: string;
        client_secret: string;
        redirect_uri: string;
      }>("get_oauth_config");

      // Initiate Google Sign-In
      const response = await signIn({
        clientId: config.client_id,
        clientSecret: config.client_secret,
        scopes: ["openid", "email", "profile"],
        redirectUri: config.redirect_uri,
      });

      userInfo = {
        idToken: response.idToken,
        accessToken: response.accessToken,
        expiresAt: response.expiresAt ? new Date(response.expiresAt) : null,
      };

      authStatus = "Successfully signed in!";
      console.log("Authentication successful:", response);
    } catch (error) {
      authStatus = `Sign-in failed: ${error}`;
      console.error("Authentication error:", error);
    } finally {
      isAuthenticating = false;
    }
  }

  async function handleGoogleSignOut() {
    try {
      await signOut();
      userInfo = null;
      authStatus = "Signed out successfully";
    } catch (error) {
      authStatus = `Sign-out failed: ${error}`;
      console.error("Sign-out error:", error);
    }
  }
</script>
