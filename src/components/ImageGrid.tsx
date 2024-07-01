import React, { useRef, useCallback, useMemo } from 'react';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import ExpandedImage from './ExpandedImage';

interface FileItem {
  name: string;
  path: string;
  is_dir: boolean;
}

interface ImageGridProps {
  files: FileItem[];
  onFileClick: (path: string) => void;
  onImageSelect: (path: string) => void;
  expandedImageIndex: number | null;
  setExpandedImageIndex: (index: number | null) => void;
}

export const ImageGrid: React.FC<ImageGridProps> = React.memo(({
  files,
  onFileClick,
  onImageSelect,
  expandedImageIndex,
  setExpandedImageIndex
}) => {
  const clickTimeoutRef = useRef<number | null>(null);
  const clickCountRef = useRef(0);

  const handleItemClick = useCallback((index: number) => {
    const file = files[index];
    clickCountRef.current += 1;

    if (clickTimeoutRef.current !== null) {
      clearTimeout(clickTimeoutRef.current);
    }

    clickTimeoutRef.current = window.setTimeout(() => {
      if (clickCountRef.current === 1) {
        // シングルクリック
        if (!file.is_dir && file.name.match(/\.(jpg|jpeg|png|gif|bmp|webp)$/i)) {
          setExpandedImageIndex(index);
        } else if (file.is_dir) {
          onFileClick(file.path);
        }
      } else if (clickCountRef.current === 2) {
        // ダブルクリック
        if (file.is_dir) {
          onFileClick(file.path);
        } else {
          onImageSelect(file.path);
        }
      }
      clickCountRef.current = 0;
    }, 200); // 200ミリ秒の遅延
  }, [files, onFileClick, onImageSelect, setExpandedImageIndex]);

  const handleNavigate = useCallback((direction: 'prev' | 'next') => {
    if (expandedImageIndex === null) return;
    
    let newIndex = expandedImageIndex + (direction === 'next' ? 1 : -1);
    
    // Skip non-image files and folders
    while (newIndex >= 0 && newIndex < files.length) {
      const file = files[newIndex];
      if (!file.is_dir && file.name.match(/\.(jpg|jpeg|png|gif|bmp|webp)$/i)) {
        break;
      }
      newIndex += direction === 'next' ? 1 : -1;
    }

    if (newIndex >= 0 && newIndex < files.length) {
      setExpandedImageIndex(newIndex);
    }
  }, [expandedImageIndex, files, setExpandedImageIndex]);

  const renderGridItem = useCallback((file: FileItem, index: number) => (
    <div
      key={file.path}
      className="aspect-square overflow-hidden rounded-lg shadow-md hover:shadow-lg transition-shadow duration-300 cursor-pointer"
      onClick={() => handleItemClick(index)}
    >
      {file.is_dir ? (
        <div className="w-full h-full flex flex-col items-center justify-center bg-gray-100">
          <svg className="w-16 h-16 text-blue-500" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
          </svg>
          <p className="mt-2 text-xs text-center text-gray-600 px-2 truncate">{file.name}</p>
        </div>
      ) : file.name.match(/\.(jpg|jpeg|png|gif|bmp|webp)$/i) ? (
        <img 
          src={convertFileSrc(file.path)} 
          alt={file.name} 
          className="w-full h-full object-cover"
        />
      ) : (
        <div className="w-full h-full flex flex-col items-center justify-center bg-gray-100">
          <svg className="w-16 h-16 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M7 21h10a2 2 0 002-2V9.414a1 1 0 00-.293-.707l-5.414-5.414A1 1 0 0012.586 3H7a2 2 0 00-2 2v14a2 2 0 002 2z" />
          </svg>
          <p className="mt-2 text-xs text-center text-gray-600 px-2 truncate">{file.name}</p>
        </div>
      )}
      {!file.is_dir && (
        <p className="absolute bottom-0 left-0 right-0 bg-black bg-opacity-50 text-white text-xs p-1 truncate">
          {file.name}
        </p>
      )}
    </div>
  ), [handleItemClick]);

  const gridItems = useMemo(() => files.map(renderGridItem), [files, renderGridItem]);

  return (
    <>
      <div className="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-5 lg:grid-cols-8 xl:grid-cols-10 gap-4">
        {gridItems}
      </div>
      {expandedImageIndex !== null && (
        <ExpandedImage
          imagePath={files[expandedImageIndex].path}
          onClose={() => setExpandedImageIndex(null)}
          onNavigate={handleNavigate}
        />
      )}
    </>
  );
});

export default ImageGrid;