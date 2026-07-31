// 主窗口入口
// PRD 12.6: 启动时 refresh notes + config，并订阅事件

import './app.css';
import { mount } from 'svelte';
import MainApp from './views/MainApp.svelte';
import { refreshNotes, startNotesSubscription } from './stores/notes';
import { refreshConfig, startConfigSubscription } from './stores/config';

const target = document.getElementById('app');
if (!target) throw new Error('挂载点 #app 未找到');

// 启动时拉取数据 + 订阅事件（不阻塞渲染，store 异步更新）
async function bootstrap() {
  await Promise.all([refreshNotes(), refreshConfig()]);
  await Promise.all([startNotesSubscription(), startConfigSubscription()]);
}

const app = mount(MainApp, { target });

bootstrap().catch((e) => console.error('[DeskNote] 启动失败', e));

export default app;
