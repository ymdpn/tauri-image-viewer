import React, { useState, useEffect } from 'react';
import ReactDOM from 'react-dom/client';
import ImageViewer from './components/ImageViewer';
import { listen } from '@tauri-apps/api/event';
import { logInfo } from './utils/logger';

const ImageViewerWindow: React.FC = () => {
  const [initialPath, setInitialPath] = useState<string | null>(null);
  const [sortBy, setSortBy] = useState<string>('name');
  const [sortOrder, setSortOrder] = useState<string>('asc');

  useEffect(() => {
    logInfo('ImageViewerWindow mounted');
    const searchParams = new URLSearchParams(window.location.search);
    const path = searchParams.get('imagePath');
    if (path) {
      logInfo('Initial path from URL:', decodeURIComponent(path));
      setInitialPath(decodeURIComponent(path));
      setSortBy(searchParams.get('sortBy') || 'name');
      setSortOrder(searchParams.get('sortOrder') || 'asc');
    }

    const unlisten = listen('init-image-viewer', (event: any) => {
      logInfo('Received init-image-viewer event:', event);
      const { initialPath, sortBy, sortOrder } = event.payload;
      setInitialPath(initialPath);
      setSortBy(sortBy);
      setSortOrder(sortOrder);
    });

    return () => {
      logInfo('ImageViewerWindow unmounting');
      unlisten.then(f => f());
    };
  }, []);

  logInfo('ImageViewerWindow render:', { initialPath, sortBy, sortOrder });

  if (!initialPath) {
    return <div>Loading...</div>;
  }

  return <ImageViewer initialPath={initialPath} sortBy={sortBy} sortOrder={sortOrder} />;
};

ReactDOM.createRoot(document.getElementById('root') as HTMLElement).render(
  <React.StrictMode>
    <ImageViewerWindow />
  </React.StrictMode>
);