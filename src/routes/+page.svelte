<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { listen } from '@tauri-apps/api/event';
  import { onMount } from "svelte";

  let cps = 10;
  let chance = 100;
  let code: string[] = [];

  const record = async () => {
    await invoke("record");
  }

  const set_cps = async () => {
    await invoke("cps", { cps: cps || 10 });
  }

  const set_chance = async () => {
    await invoke("chance", { chance: chance || 100 });
  }

  listen<string[]>("recorded", e => {
    code = e.payload;
  });

  onMount(async () => {
    code = await invoke("bind");
  });
</script>

<div class="container">
  <p>Cps</p>
  <input type="number" bind:value={cps} on:change={set_cps} placeholder="CPS"/>
  <p>Chance (%)</p>
  <input type="number" bind:value={chance} on:change={set_chance} placeholder="Chance"/>
  <p>Bind</p>
  <div class="bind">
    {code.join(" + ")}
  </div>
  <button on:click={record}>Record Bind</button>
</div>

<style>
  :root {
    font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
    font-size: 16px;
    line-height: 24px;
    font-weight: 400;

    color: #0f0f0f;
    background-color: #f6f6f6;

    font-synthesis: none;
    text-rendering: optimizeLegibility;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    -webkit-text-size-adjust: 100%;
  }

  .container {
    margin: 0;
    display: flex;
    flex-direction: column;
    justify-content: center;
    text-align: center;
  }

  p {
    margin: .5rem;
    text-align: left;
  }

  .bind {
    margin-bottom: .5rem;
    text-wrap: nowrap;
  }

  .bind, input {
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    border: none;
    padding: 0.6em 1.2em;
    border-radius: 8px;
  }

  button {
    border-radius: 8px;
    border: 1px solid transparent;
    padding: 0.6em 1.2em;
    font-size: 1em;
    font-weight: 500;
    font-family: inherit;
    color: #0f0f0f;
    background-color: #ffffff;
    transition: border-color 0.25s;
    box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
  }

  button {
    cursor: pointer;
  }

  button:hover {
    border-color: #396cd8;
  }
  button:active {
    border-color: #396cd8;
    background-color: #e8e8e8;
  }

  @media (prefers-color-scheme: dark) {
    :root {
      color: #f6f6f6;
      background-color: #2f2f2f;
    }

    .bind, input {
      color: #ffffff;
      background-color: #18181898;
    }

    button {
      color: #ffffff;
      background-color: #0f0f0f98;
    }
    button:active {
      background-color: #0f0f0f69;
    }
  }
</style>
