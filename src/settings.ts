// 设置窗口入口（SOP 7.2）
import './app.css';
import { mount } from 'svelte';
import Settings from './views/Settings.svelte';
import { refreshConfig, startConfigSubscription } from './stores/config';

const target = document.getElementById('app');
if (!target) throw new Error('挂载点 #app 未找到');

async function bootstrap() {
  await refreshConfig();
  await startConfigSubscription();
}

const app = mount(Settings, { target });

bootstrap().catch((e) => console.error('[DeskNote] 设置窗口启动失败', e));

export default app;
