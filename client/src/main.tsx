import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { WorldPreview } from './preview/WorldPreview';
import './shell.css';

const element = document.getElementById('root');
if (element) {
  const preview = new URLSearchParams(location.search).get('world-preview') === '1';
  if (preview) {
    const icon = document.createElement('link');
    icon.rel = 'icon';
    icon.href = '/preview/world/favicon.svg';
    document.head.append(icon);
  }
  createRoot(element).render(<StrictMode>{preview ? <WorldPreview /> : <App />}</StrictMode>);
}
