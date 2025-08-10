<script>
  import Header from './components/Header.svelte';
  import Overview from './components/Overview.svelte';
  import SessionReplay from './components/SessionReplay.svelte';
  import IssuesAndDiagnostics from './components/IssuesAndDiagnostics.svelte';
  import FeatureManagement from './components/FeatureManagement.svelte';
  import WorkspaceIDE from './components/WorkspaceIDE.svelte';
  import TestingQuality from './components/TestingQuality.svelte';
  import EntityExplorer from './components/EntityExplorer.svelte';
  import AnalyticsInsights from './components/AnalyticsInsights.svelte';
  import Settings from './components/Settings.svelte';
  
  let currentView = 'overview';
  
  function switchView(view) {
    currentView = view;
  }
</script>

<main>
  <Header {currentView} {switchView} />
  
  <div class="content">
    {#if currentView === 'overview'}
      <Overview />
    {:else if currentView === 'sessions'}
      <SessionReplay />
    {:else if currentView === 'issues'}
      <IssuesAndDiagnostics />
    {:else if currentView === 'features'}
      <FeatureManagement />
    {:else if currentView === 'workspace'}
      <WorkspaceIDE />
    {:else if currentView === 'testing'}
      <TestingQuality />
    {:else if currentView === 'entities'}
      <EntityExplorer />
    {:else if currentView === 'analytics'}
      <AnalyticsInsights />
    {:else if currentView === 'settings'}
      <Settings />
    {:else}
      <div class="other-view">
        <h1>{currentView.charAt(0).toUpperCase() + currentView.slice(1)}</h1>
        <p>This section is under development</p>
      </div>
    {/if}
  </div>
</main>

<style>
  :global(html) {
    color-scheme: light dark;
  }

  :global(body) {
    margin: 0;
    font-family: system-ui, sans-serif;
  }

  main {
    min-height: 100vh;
  }

  .content {
    max-width: 1400px;
    margin: 0 auto;
    padding: var(--spacing-xl);
  }

  .other-view {
    padding: 2rem;
    text-align: center;
  }
</style>