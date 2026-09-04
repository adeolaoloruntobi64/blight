import { lazy, Suspense } from 'react'
import ReactDOM from 'react-dom/client'
import { BrowserRouter, Routes, Route } from 'react-router'
import App from './App'
import './index.css'
import { loadScramjetScripts } from './blight/load-scripts'


const BlightApp = lazy(async () => {
  await loadScramjetScripts();
  return await import("./blight/BlightApp");
});

ReactDOM.createRoot(document.getElementById('root')!).render(
    <BrowserRouter>
      <Suspense fallback={<div>Loading page…</div>}>
        <Routes>
          <Route path="/" element={<App />} />
          <Route path="/blight/" element={<BlightApp />} />
        </Routes>
      </Suspense>
    </BrowserRouter>
)
