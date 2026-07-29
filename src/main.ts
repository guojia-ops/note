// 主窗口入口（阶段 1 占位）
import './app.css';
import { mount } from 'svelte';
import MainApp from './views/MainApp.svelte';

const target = document.getElementById('app');
if (!target) throw new Error('挂载点 #app 未找到');

const app = mount(MainApp, { target });

export default app;
