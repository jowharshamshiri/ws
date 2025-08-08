<script>
  import Header from './components/Header.svelte';
  import Overview from './components/Overview.svelte';

  let currentView = 'overview';

  function switchView(view) {
    currentView = view;
  }
</script>

<div class="ade-app">
  <Header {currentView} {switchView} />
  
  <div class="ade-content">
    {#if currentView === 'overview'}
      <div class="overview-grid">
        <Overview />
      </div>
    {:else}
      <div class="other-view">
        <h1>{currentView.charAt(0).toUpperCase() + currentView.slice(1)}</h1>
        <p>This section is under development</p>
      </div>
    {/if}
  </div>
</div>

<style>
  .ade-app {
    min-height: 100vh;
    background: #0a0a0b;
    color: #ffffff;
    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
  }

  .ade-content {
    max-width: 1400px;
    margin: 0 auto;
    padding: 20px;
  }

  .overview-grid {
    margin-top: 20px;
  }

  .other-view {
    padding: 32px;
    text-align: center;
  }

  .other-view h1 {
    font-size: 24px;
    font-weight: 600;
    color: #111827;
    margin-bottom: 8px;
  }

  .other-view p {
    color: #6b7280;
    font-size: 16px;
  }
</style>