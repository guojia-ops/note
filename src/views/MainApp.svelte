<script lang="ts">
  // 主窗口组件（阶段 1 占位，阶段 6 实现完整功能）
</script>

<main class="main-app">
  <h1>DeskNote</h1>
  <p>轻量级桌面便签应用</p>
  <p class="placeholder">主窗口占位 — 阶段 6 将实现便签网格墙</p>
</main>

<style>
  .main-app {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    height: 100%;
    padding: 2rem;
    text-align: center;
  }

  h1 {
    font-size: 2rem;
    font-weight: 600;
    margin-bottom: 0.5rem;
    color: #f9a825;
  }

  p {
    color: #666;
    margin-bottom: 0.25rem;
  }

  .placeholder {
    margin-top: 2rem;
    font-size: 0.875rem;
    color: #999;
    padding: 0.5rem 1rem;
    border: 1px dashed #ccc;
    border-radius: 4px;
  }
</style>
