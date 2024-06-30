import React, { useState, useEffect, useCallback } from 'react';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { invoke } from '@tauri-apps/api/tauri';
import { logInfo, logError } from '../utils/logger';

interface ImageViewerProps {
  initialPath: string;
  sortBy: string;
  sortOrder: string;
}

const ImageViewer: React.FC<ImageViewerProps> = ({ initialPath, sortBy, sortOrder }) => {
  const [currentImagePath, setCurrentImagePath] = useState<string | null>(null);
  const [fullImageList, setFullImageList] = useState<string[]>([]);
  const [currentIndex, setCurrentIndex] = useState<number>(0);

  const loadImageList = useCallback(async (path: string) => {
    try {
      logInfo('Loading image list with:', { path, sortBy, sortOrder });
      const result = await invoke<string[]>('get_full_image_list', { 
        path,
        sortBy,
        sortOrder
      });
      logInfo('Received image list:', result);
      setFullImageList(result);
      const selectedIndex = result.findIndex(imgPath => imgPath === path);
      logInfo('Selected index:', selectedIndex);
      setCurrentIndex(selectedIndex !== -1 ? selectedIndex : 0);
      setCurrentImagePath(convertFileSrc(path));
    } catch (error) {
      logError('Error loading image list:', error);
    }
  }, [sortBy, sortOrder]);

  useEffect(() => {
    loadImageList(initialPath);
  }, [initialPath, loadImageList]);

  const navigateImage = useCallback((direction: 'next' | 'prev') => {
    logInfo('Navigating image:', direction, 'Current index:', currentIndex);
    let newIndex = direction === 'next' ? currentIndex + 1 : currentIndex - 1;
    
    if (newIndex < 0) {
      newIndex = fullImageList.length - 1;
    } else if (newIndex >= fullImageList.length) {
      newIndex = 0;
    }

    logInfo('New index:', newIndex, 'Full list length:', fullImageList.length);
    setCurrentIndex(newIndex);
    const newPath = fullImageList[newIndex];
    logInfo('New image path:', newPath);
    setCurrentImagePath(convertFileSrc(newPath));
  }, [currentIndex, fullImageList]);

  useEffect(() => {
    loadImageList(initialPath);
  }, [initialPath, loadImageList]);

  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === 'ArrowRight') {
        navigateImage('next');
      } else if (e.key === 'ArrowLeft') {
        navigateImage('prev');
      }
    };

    const handleWheel = (e: WheelEvent) => {
      console.log("wheel")
      e.preventDefault();
      if (e.deltaY > 0) {
        navigateImage('next');
      } else {
        navigateImage('prev');
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('wheel', handleWheel, { passive: false });

    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('wheel', handleWheel);
    };
  }, [navigateImage]);

  console.log('Current state:', { currentIndex, fullImageList: fullImageList.length, currentImagePath });

  if (!currentImagePath) {
    return <div>Loading...</div>;
  }

  return (
    <div style={{ width: '100%', height: '100%', display: 'flex', justifyContent: 'center', alignItems: 'center' }}>
      <img src={currentImagePath} alt="Viewed image" style={{ maxWidth: '100%', maxHeight: '100%', objectFit: 'contain' }} />
    </div>
  );
};

export default ImageViewer;