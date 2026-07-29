// 便签窗口入口（阶段 1 占位）
import './app.css';
import { mount } from 'svelte';
import NoteApp from './views/NoteApp.svelte';

const target = document.getElementById('app');
if (!target) throw new Error('挂载点 #app 未找到');

const app = mount(NoteApp, { target });

export default app;
