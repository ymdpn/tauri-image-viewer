import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/tauri';

interface FolderTreeProps {
  onFolderSelect: (path: string) => void;
  selectedPath: string | null;
}

interface TreeNode {
  id: string;
  name: string;
  path: string;
  children: TreeNode[];
  isExpanded: boolean;
}

export const FolderTree: React.FC<FolderTreeProps> = ({ onFolderSelect, selectedPath }) => {
  const [tree, setTree] = useState<TreeNode[]>([]);

  useEffect(() => {
    loadRootFolders();
  }, []);

  const loadRootFolders = async () => {
    try {
      const rootFolders = await invoke<{ id: string; name: string; path: string }[]>('get_root_folders');
      setTree(rootFolders.map(folder => ({
        ...folder,
        children: [],
        isExpanded: false
      })));
    } catch (error) {
      console.error('Error loading root folders:', error);
    }
  };

  const toggleFolder = async (node: TreeNode) => {
    if (!node.isExpanded) {
      try {
        const children = await invoke<{ id: string; name: string; path: string; is_dir: boolean }[]>('get_directory_contents', { path: node.path });
        node.children = children
          .filter(child => child.is_dir)
          .map(child => ({ ...child, children: [], isExpanded: false }));
      } catch (error) {
        console.error('Error loading subfolder:', error);
      }
    }
    node.isExpanded = !node.isExpanded;
    setTree([...tree]);
  };

  const renderTree = (nodes: TreeNode[]) => (
    <ul className="pl-4">
      {nodes.map(node => (
        <li key={node.path} className="py-1">
          <div className={`flex items-center rounded-md ${selectedPath === node.path ? 'bg-blue-100' : 'hover:bg-gray-100'}`}>
            <button onClick={() => toggleFolder(node)} className="mr-1 focus:outline-none p-1">
              <svg className={`w-4 h-4 text-gray-500 transform transition-transform ${node.isExpanded ? 'rotate-90' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </button>
            <svg className="w-5 h-5 text-yellow-500 mr-1" fill="none" stroke="currentColor" viewBox="0 0 24 24" xmlns="http://www.w3.org/2000/svg">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
            </svg>
            <span 
              onClick={() => onFolderSelect(node.path)} 
              className={`cursor-pointer truncate py-1 px-2 rounded-md flex-grow ${selectedPath === node.path ? 'text-blue-600 font-semibold' : 'hover:text-blue-500'}`}
            >
              {node.name}
            </span>
          </div>
          {node.isExpanded && node.children.length > 0 && renderTree(node.children)}
        </li>
      ))}
    </ul>
  );

  return (
    <div className="folder-tree h-full flex flex-col">
      <h3 className="text-lg font-semibold mb-2">Folders</h3>
      <div className="overflow-auto flex-grow">
        {renderTree(tree)}
      </div>
    </div>
  );
};