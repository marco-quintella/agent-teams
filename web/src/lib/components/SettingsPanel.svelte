<script lang="ts">
  import { onMount } from 'svelte';
  import { api, type ClaudeSettingsView, type DoctorResponse } from '../api/client';

  let doctor = $state<DoctorResponse | null>(null);
  let settings = $state<ClaudeSettingsView | null>(null);
  let credentialMode = $state<'cli_login' | 'api_key'>('cli_login');
  let apiKey = $state('');
  let apiBaseUrl = $state('');
  let busy = $state(false);
  let message = $state('');
  let error = $state('');

  async function refresh() {
    error = '';
    try {
      doctor = await api.doctor();
      settings = await api.getClaudeSettings();
      credentialMode = settings.credential_mode as 'cli_login' | 'api_key';
      apiBaseUrl = settings.api_base_url ?? '';
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    }
  }

  onMount(refresh);

  async function saveSettings() {
    busy = true;
    error = '';
    message = '';
    try {
      settings = await api.patchClaudeSettings({
        credential_mode: credentialMode,
        api_key: credentialMode === 'api_key' && apiKey.trim() ? apiKey.trim() : undefined,
        api_base_url: apiBaseUrl.trim() || undefined,
      });
      apiKey = '';
      message = 'Settings saved.';
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function runCliLogin() {
    busy = true;
    error = '';
    message = '';
    try {
      const res = await api.claudeLogin();
      message = res.message;
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }

  async function installCli() {
    if (!confirm('Install Claude Code CLI using the platform installer (winget on Windows)?')) {
      return;
    }
    busy = true;
    error = '';
    message = '';
    try {
      const res = await api.installClaude(true);
      message = res.ok
        ? `Install finished: ${res.command}`
        : `Install attempted (${res.command}). Check output in server logs.`;
      if (res.output) {
        message += `\n${res.output.slice(0, 500)}`;
      }
      await refresh();
    } catch (e) {
      error = e instanceof Error ? e.message : String(e);
    } finally {
      busy = false;
    }
  }
</script>

<section class="panel">
  <h2>Settings — Claude Code</h2>

  {#if doctor}
    <div class="doctor" class:ok={doctor.credentials.ready && doctor.cli.found}>
      <h3>Doctor</h3>
      <p>orchestrator v{doctor.orchestrator_version}</p>
      <p>
        CLI: {doctor.cli.found ? doctor.cli.version ?? 'found' : 'not found'}
      </p>
      <p>
        Credentials ({doctor.credentials.mode}):
        {doctor.credentials.ready ? 'ready' : 'not ready'}
      </p>
      <p class="hint">{doctor.credentials.hint}</p>
      {#if !doctor.cli.found}
        <button type="button" disabled={busy} onclick={installCli}>Install Claude CLI</button>
      {/if}
    </div>
  {/if}

  <form
    class="form"
    onsubmit={(e) => {
      e.preventDefault();
      saveSettings();
    }}
  >
    <label>
      <span>Credential mode</span>
      <select bind:value={credentialMode} disabled={busy}>
        <option value="cli_login">CLI login (OAuth / Anthropic account)</option>
        <option value="api_key">API key (incl. OpenRouter)</option>
      </select>
    </label>

    {#if credentialMode === 'api_key'}
      <label>
        <span>API key</span>
        <input
          type="password"
          bind:value={apiKey}
          placeholder={settings?.api_key_masked ?? 'sk-…'}
          disabled={busy}
        />
      </label>
      <label>
        <span>API base URL (optional, e.g. OpenRouter)</span>
        <input
          type="url"
          bind:value={apiBaseUrl}
          placeholder="https://openrouter.ai/api"
          disabled={busy}
        />
      </label>
    {:else}
      <button type="button" class="secondary" disabled={busy} onclick={runCliLogin}>
        Run CLI login
      </button>
    {/if}

    <button type="submit" disabled={busy}>Save</button>
  </form>

  {#if message}
    <p class="msg">{message}</p>
  {/if}
  {#if error}
    <p class="err">{error}</p>
  {/if}
</section>

<style>
  .panel {
    background: #161a22;
    border: 1px solid #2a3140;
    border-radius: 8px;
    padding: 1rem;
  }
  h2 {
    margin: 0 0 0.75rem;
    font-size: 1rem;
  }
  h3 {
    margin: 0 0 0.5rem;
    font-size: 0.9rem;
  }
  .doctor {
    background: #2a2218;
    border-radius: 6px;
    padding: 0.75rem;
    margin-bottom: 1rem;
    font-size: 0.85rem;
  }
  .doctor.ok {
    background: #1a2a22;
  }
  .doctor p {
    margin: 0.25rem 0;
  }
  .hint {
    color: #a8b4c8;
  }
  .form {
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
  }
  label {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    font-size: 0.85rem;
  }
  input,
  select {
    padding: 0.4rem 0.5rem;
    border-radius: 4px;
    border: 1px solid #3a4458;
    background: #0d0f12;
    color: #e8ecf1;
  }
  button {
    padding: 0.45rem 0.75rem;
    border-radius: 4px;
    border: none;
    background: #3d6ae8;
    color: #fff;
    cursor: pointer;
  }
  button:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  button.secondary {
    background: #3a4458;
  }
  .msg {
    color: #90c090;
    font-size: 0.85rem;
  }
  .err {
    color: #f5a0a0;
    font-size: 0.85rem;
  }
</style>
