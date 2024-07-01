import { useState, useEffect, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { WebviewWindow, getCurrent } from '@tauri-apps/api/window';
import { convertFileSrc } from '@tauri-apps/api/tauri';
import { FolderTree } from './components/FolderTree';
import { ImageGrid } from './components/ImageGrid';
import { SortControls } from './components/SortControls';

interface FileItem {
  name: string;
  path: string;
  is_dir: boolean;
  date_modified: number;
  size: number;
}

interface StartupInfo {
  folder: string;
  file: string | null;
}

function App() {
  const [currentPath, setCurrentPath] = useState<string | null>(null);
  const [files, setFiles] = useState<FileItem[]>([]);
  const [sortBy, setSortBy] = useState<'name' | 'type' | 'date' | 'size'>('type');
  const [sortOrder, setSortOrder] = useState<'asc' | 'desc'>('asc');
  const [isCloneWindow, setIsCloneWindow] = useState(false);
  const [selectedImagePath, setSelectedImagePath] = useState<string | null>(null);
  const [fullImageList, setFullImageList] = useState<string[]>([]);
  const [expandedImageIndex, setExpandedImageIndex] = useState<number | null>(null);
  const [zoomLevel, setZoomLevel] = useState(1);

  useEffect(() => {
    const searchParams = new URLSearchParams(window.location.search);
    const isClone = searchParams.get('clone') === 'true';
    setIsCloneWindow(isClone);

    if (isClone) {
      const imagePath = searchParams.get('imagePath');
      if (imagePath) {
        setSelectedImagePath(imagePath);
        loadImageList(imagePath);
      }
    } else {
      initializeApp();
    }
  }, []);
  

  useEffect(() => {
    if (isCloneWindow) {
      window.addEventListener('wheel', handleWheel);
      window.addEventListener('keydown', handleKeyDown);
      return () => {
        window.removeEventListener('wheel', handleWheel);
        window.removeEventListener('keydown', handleKeyDown);
      };
    }
  }, [isCloneWindow, expandedImageIndex, fullImageList, zoomLevel]);

  useEffect(() => {
    if (currentPath) {
      loadDirectory(currentPath);
      invoke('save_last_folder', { folder: currentPath });
    }
  }, [currentPath, sortBy, sortOrder]);

  const initializeApp = useCallback(async () => {
    try {
      const startupInfo: StartupInfo = await invoke('get_startup_info');
      setCurrentPath(startupInfo.folder);
      
      if (startupInfo.file) {
        setSelectedImagePath(startupInfo.file);
        
        // ファイルリストとイメージリストを並行して取得
        const [files, imageList] = await Promise.all([
          invoke<FileItem[]>('get_directory_contents', { path: startupInfo.folder }),
          invoke<string[]>('get_full_image_list', { 
            path: startupInfo.file,
            sortBy: sortBy.toLowerCase(),
            sortOrder: sortOrder.toLowerCase()
          })
        ]);
        
        setFiles(sortFiles(files));
        setFullImageList(imageList);
        
        const index = imageList.findIndex(path => path === startupInfo.file);
        if (index !== -1) {
          setExpandedImageIndex(index);
        }
      } else {
        loadDirectory(startupInfo.folder);
      }
    } catch (error) {
      console.error('Error getting startup info:', error);
      selectFolder();
    }
  }, [sortBy, sortOrder]);

  useEffect(() => {
    initializeApp();
  }, [initializeApp]);

  const selectFolder = async () => {
    try {
      const selectedPath = await invoke<string>('select_folder');
      setCurrentPath(selectedPath);
    } catch (error) {
      console.error('Error selecting folder:', error);
    }
  };

  const loadDirectory = async (path: string) => {
    try {
      const result = await invoke<FileItem[]>('get_directory_contents', { path });
      const sortedFiles = sortFiles(result);
      setFiles(sortedFiles);
      return sortedFiles;
    } catch (error) {
      console.error('Error loading directory:', error);
      return [];
    }
  };

  const loadImageList = async (imagePath: string) => {
    try {
      const result = await invoke<string[]>('get_full_image_list', { 
        path: imagePath,
        sortBy: sortBy.toLowerCase(),
        sortOrder: sortOrder.toLowerCase()
      });
      setFullImageList(result);
      const selectedIndex = result.findIndex(path => path === imagePath);
      setExpandedImageIndex(selectedIndex !== -1 ? selectedIndex : 0);
      setSelectedImagePath(imagePath);
    } catch (error) {
      console.error('Error loading image list:', error);
    }
  };

  const sortFiles = useCallback((files: FileItem[]) => {
    return [...files].sort((a, b) => {
      if (a.is_dir !== b.is_dir) {
        return a.is_dir ? -1 : 1;
      }
      switch (sortBy) {
        case 'name':
          return sortOrder === 'asc' ? a.name.localeCompare(b.name) : b.name.localeCompare(a.name);
        case 'type':
          const getFileExtension = (filename: string) => filename.slice((filename.lastIndexOf(".") - 1 >>> 0) + 2);
          const extA = getFileExtension(a.name);
          const extB = getFileExtension(b.name);
          return sortOrder === 'asc' ? extA.localeCompare(extB) : extB.localeCompare(extA);
        case 'date':
          return sortOrder === 'asc' ? a.date_modified - b.date_modified : b.date_modified - a.date_modified;
        case 'size':
          return sortOrder === 'asc' ? a.size - b.size : b.size - a.size;
        default:
          return 0;
      }
    });
  }, [sortBy, sortOrder]);

  const memoizedSortedFiles = useMemo(() => sortFiles(files), [files, sortFiles]);

  const handleWheel = useCallback((e: WheelEvent) => {
    e.preventDefault();
    if (e.deltaY > 0) {
      navigateImage('next');
    } else {
      navigateImage('prev');
    }
  }, [expandedImageIndex, fullImageList]);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'ArrowRight') {
      navigateImage('next');
    } else if (e.key === 'ArrowLeft') {
      navigateImage('prev');
    } else if (e.key === 'ArrowUp') {
      setZoomLevel(prevZoom => Math.min(prevZoom * 1.1, 3));
    } else if (e.key === 'ArrowDown') {
      setZoomLevel(prevZoom => Math.max(prevZoom / 1.1, 0.1));
    }
  }, [expandedImageIndex, fullImageList]);

  const navigateImage = (direction: 'next' | 'prev') => {
    if (expandedImageIndex === null) return;
    
    let newIndex = direction === 'next' ? expandedImageIndex + 1 : expandedImageIndex - 1;
    
    if (newIndex < 0) {
      newIndex = fullImageList.length - 1;
    } else if (newIndex >= fullImageList.length) {
      newIndex = 0;
    }

    setExpandedImageIndex(newIndex);
    setSelectedImagePath(fullImageList[newIndex]);
    setZoomLevel(1);
  };

  const handleFolderSelect = (path: string) => {
    setCurrentPath(path);
  };

  const handleImageSelect = async (path: string) => {
    console.log('handleImageSelect called with path:', path);
    try {
      const currentWindow = getCurrent();
      const currentPosition = await currentWindow.innerPosition();
      const currentSize = await currentWindow.innerSize();

      await loadImageList(path);

      const cloneWindow = new WebviewWindow(`image-${Date.now()}`, {
        url: `${window.location.origin}?clone=true&imagePath=${encodeURIComponent(path)}&sortBy=${sortBy}&sortOrder=${sortOrder}`,
        title: `Image Viewer`,
        x: currentPosition.x + 50,
        y: currentPosition.y + 50,
        width: currentSize.width,
        height: currentSize.height,
      });

      await cloneWindow.once('tauri://created', () => {
        console.log('Clone window created');
        cloneWindow.emit('init-image-viewer', { 
          initialPath: path,
          fullImageList: fullImageList,
          sortBy,
          sortOrder
        });
      });
    } catch (error) {
      console.error('Error opening clone window:', error);
    }
  };

  const handleSortByChange = (newSortBy: 'name' | 'type' | 'date' | 'size') => {
    setSortBy(newSortBy);
  };

  const handleSortOrderChange = (newSortOrder: 'asc' | 'desc') => {
    setSortOrder(newSortOrder);
  };

  return (
    <div className="flex h-screen bg-gray-100">
      {!isCloneWindow && (
        <>
          <div className="w-1/4 p-4 bg-white shadow-md flex flex-col h-full">
            <button 
              onClick={selectFolder}
              className="w-full mb-4 px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600 transition-colors"
            >
              Select Folder
            </button>
            <div className="overflow-auto flex-grow">
              <FolderTree onFolderSelect={handleFolderSelect} selectedPath={currentPath} />
            </div>
          </div>
          <div className="w-3/4 p-4 overflow-auto">
            <div className="mb-4">
              <SortControls
                sortBy={sortBy}
                sortOrder={sortOrder}
                onSortByChange={handleSortByChange}
                onSortOrderChange={handleSortOrderChange}
              />
            </div>
            {currentPath ? (
              <ImageGrid
                files={memoizedSortedFiles}
                onFileClick={handleFolderSelect}
                onImageSelect={handleImageSelect}
                expandedImageIndex={expandedImageIndex}
                setExpandedImageIndex={setExpandedImageIndex}
              />
            ) : (
              <p className="text-center text-gray-500 mt-10">Please select a folder to start.</p>
            )}
          </div>
        </>
      )}
      {isCloneWindow && selectedImagePath && (
        <div style={{ 
          width: '100%', 
          height: '100%', 
          display: 'flex', 
          justifyContent: 'center', 
          alignItems: 'center',
          overflow: 'hidden'
        }}>
          <img 
            src={convertFileSrc(selectedImagePath)}
            alt="Selected image" 
            style={{ 
              maxWidth: '100%', 
              maxHeight: '100%', 
              objectFit: 'contain',
              transform: `scale(${zoomLevel})`,
              transition: 'transform 0.2s ease-out'
            }} 
          />
        </div>
      )}
    </div>
  );
}

export default App;