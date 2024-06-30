// src/components/SortControls.tsx

import React from 'react';

interface SortControlsProps {
  sortBy: 'name' | 'type' | 'date' | 'size';
  sortOrder: 'asc' | 'desc';
  onSortByChange: (sortBy: 'name' | 'type' | 'date' | 'size') => void;
  onSortOrderChange: (sortOrder: 'asc' | 'desc') => void;
}

export const SortControls: React.FC<SortControlsProps> = ({
  sortBy,
  sortOrder,
  onSortByChange,
  onSortOrderChange
}) => {
  return (
    <div className="flex items-center space-x-4">
      <select
        value={sortBy}
        onChange={(e) => onSortByChange(e.target.value as 'name' | 'type' | 'date' | 'size')}
        className="px-3 py-2 bg-white border border-gray-300 rounded-md shadow-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        <option value="name">Name</option>
        <option value="type">Type</option>
        <option value="date">Date</option>
        <option value="size">Size</option>
      </select>
      <button
        onClick={() => onSortOrderChange(sortOrder === 'asc' ? 'desc' : 'asc')}
        className="px-3 py-2 bg-white border border-gray-300 rounded-md shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-blue-500"
      >
        {sortOrder === 'asc' ? '↑' : '↓'}
      </button>
    </div>
  );
};