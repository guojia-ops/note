// 便签窗口入口
// 阶段 5：从 URL query 解析 id，挂载 NoteApp
// 阶段 9：NoteApp 内部订阅 config:updated，主题变更实时同步

import './app.css';
import { mount } from 'svelte';
import NoteApp from './views/NoteApp.svelte';

const target = document.getElementById('app');
if (!target) throw new Error('挂载点 #app 未找到');

const app = mount(NoteApp, { target });

export default app;
