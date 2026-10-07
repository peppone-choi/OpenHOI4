import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { App } from './App';
import { WorldPreview } from './preview/WorldPreview';
import './shell.css';

const element = document.getElementById('root');
if (element) {
  const preview = new URLSearchParams(location.search).get('world-preview') === '1';
  createRoot(element).render(<StrictMode>{preview ? <WorldPreview /> : <App />}</StrictMode>);
}
