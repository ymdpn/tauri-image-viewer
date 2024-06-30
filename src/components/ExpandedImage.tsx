import React, { useState, useEffect, useCallback } from 'react';
import { convertFileSrc } from '@tauri-apps/api/tauri';

interface ExpandedImageProps {
  imagePath: string;
  onClose: () => void;
  onNavigate: (direction: 'prev' | 'next') => void;
}

const ExpandedImage: React.FC<ExpandedImageProps> = ({ imagePath, onClose, onNavigate }) => {
  const [zoomLevel, setZoomLevel] = useState(1);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowLeft':
        onNavigate('prev');
        break;
      case 'ArrowRight':
        onNavigate('next');
        break;
      case 'ArrowUp':
        setZoomLevel(prevZoom => Math.min(prevZoom * 1.1, 3));
        break;
      case 'ArrowDown':
        setZoomLevel(prevZoom => Math.max(prevZoom / 1.1, 0.1));
        break;
      case 'Escape':
        onClose();
        break;
    }
  }, [onNavigate, onClose]);

  const handleWheel = useCallback((e: WheelEvent) => {
    e.preventDefault();
    if (e.deltaY > 0) {
      onNavigate('next');
    } else {
      onNavigate('prev');
    }
  }, [onNavigate]);

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('wheel', handleWheel, { passive: false });
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      window.removeEventListener('wheel', handleWheel);
    };
  }, [handleKeyDown, handleWheel]);

  return (
    <div className="fixed inset-0 bg-black bg-opacity-75 flex items-center justify-center z-50" onClick={onClose}>
      <div className="max-w-full max-h-full p-4 overflow-hidden">
        <img 
          src={convertFileSrc(imagePath)} 
          alt="Expanded view" 
          className="max-w-full max-h-full object-contain transition-transform duration-200"
          style={{ transform: `scale(${zoomLevel})` }}
          onClick={(e) => e.stopPropagation()}
        />
      </div>
      <button 
        className="absolute top-4 right-4 text-white text-2xl"
        onClick={onClose}
      >
        ×
      </button>
      <div className="absolute bottom-4 left-4 right-4 text-white text-center">
        Use arrow keys to navigate and zoom. Scroll to change images.
      </div>
    </div>
  );
};

export default ExpandedImage;