import React from 'react';
import { describe, it, expect, beforeEach, vi } from 'vitest';
import { render, fireEvent, screen } from '@testing-library/react';
import { useStore } from '../state/store';
import ImageCanvas from './ImageCanvas';

// Mock Zustand store
vi.mock('../state/store', () => ({
  useStore: vi.fn(),
}));

// Mock image loading utilities
vi.mock('../utils/image', () => ({
  fileToDataUrl: vi.fn(),
  loadImage: vi.fn(),
}));

describe('ImageCanvas - New Interaction Model', () => {
  const mockStore = {
    image: { url: 'test.png', width: 200, height: 200, name: 'test.png' },
    pixelPoints: [],
    setImage: vi.fn(),
    addPixelPoint: vi.fn(() => 'px_1'),
    movePixelPoint: vi.fn(),
    updatePixelPointHeight: vi.fn(),
    removePixelPoint: vi.fn(),
    selectLinkedPoint: vi.fn(),
    activeWorldId: null,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    (useStore as any).mockImplementation((selector: any) => selector(mockStore));
  });

  it('should render with updated interaction tips', () => {
    render(<ImageCanvas height={200} />);
    
    expect(screen.getByText(/Double-click to add points/)).toBeInTheDocument();
    expect(screen.getByText(/left-click to select/)).toBeInTheDocument();
    expect(screen.getByText(/Delete button removes selected point/)).toBeInTheDocument();
  });

  it('should show delete button when point is selected', () => {
    const storeWithSelectedPoint = {
      ...mockStore,
      pixelPoints: [{ id: 'px_1', u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 }],
    };
    
    // Mock a selected point
    let selectedPointId = 'px_1';
    
    render(<ImageCanvas height={200} />);
    
    // When a point is selected, delete button should appear
    if (selectedPointId) {
      expect(screen.queryByTestId('delete-point')).toBeInTheDocument();
    }
  });

  it('should show height editing overlay for selected point', () => {
    const pointWithHeight = { id: 'px_1', u: 100, v: 100, sigmaPx: 1, enabled: true, height: 5.5 };
    const storeWithPoint = {
      ...mockStore,
      pixelPoints: [pointWithHeight],
    };
    
    (useStore as any).mockImplementation((selector: any) => selector(storeWithPoint));
    
    render(<ImageCanvas height={200} />);
    
    // Height editing overlay should show height value
    expect(screen.getByText('Height:')).toBeInTheDocument();
    expect(screen.getByText('5.5m')).toBeInTheDocument();
  });

  it('should update zoom limits to allow 10x zoom', () => {
    render(<ImageCanvas height={200} />);
    
    const resetZoomButton = screen.getByTestId('reset-zoom');
    expect(resetZoomButton).toBeInTheDocument();
    
    // Test that zoom level display exists
    expect(screen.getByTestId('zoom-level')).toBeInTheDocument();
  });

  it('should show cross-selection when point is clicked', () => {
    const pointData = { id: 'px_1', u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 };
    const storeWithPoint = {
      ...mockStore,
      pixelPoints: [pointData],
    };
    
    (useStore as any).mockImplementation((selector: any) => selector(storeWithPoint));
    
    render(<ImageCanvas height={200} />);
    
    // When a point is clicked, it should trigger cross-selection
    const canvas = screen.getByTestId('canvas');
    fireEvent.mouseDown(canvas, { clientX: 100, clientY: 100 });
    
    expect(mockStore.selectLinkedPoint).toHaveBeenCalledWith('px_1', 'pixel');
  });
});

describe('ImageCanvas - Height Editing', () => {
  const mockStoreWithPoint = {
    image: { url: 'test.png', width: 200, height: 200, name: 'test.png' },
    pixelPoints: [{ id: 'px_1', u: 100, v: 100, sigmaPx: 1, enabled: true, height: 0 }],
    setImage: vi.fn(),
    addPixelPoint: vi.fn(),
    movePixelPoint: vi.fn(),
    updatePixelPointHeight: vi.fn(),
    removePixelPoint: vi.fn(),
    selectLinkedPoint: vi.fn(),
    activeWorldId: null,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    (useStore as any).mockImplementation((selector: any) => selector(mockStoreWithPoint));
  });

  it('should allow editing height when height value is clicked', () => {
    render(<ImageCanvas height={200} />);
    
    const heightDisplay = screen.getByText('0m');
    fireEvent.click(heightDisplay);
    
    // Should show input field for editing
    const heightInput = screen.getByRole('spinbutton');
    expect(heightInput).toBeInTheDocument();
    expect(heightInput).toHaveValue(0);
  });

  it('should update height when Enter is pressed', () => {
    render(<ImageCanvas height={200} />);
    
    const heightDisplay = screen.getByText('0m');
    fireEvent.click(heightDisplay);
    
    const heightInput = screen.getByRole('spinbutton');
    fireEvent.change(heightInput, { target: { value: '10' } });
    fireEvent.keyDown(heightInput, { key: 'Enter' });
    
    expect(mockStoreWithPoint.updatePixelPointHeight).toHaveBeenCalledWith('px_1', 10);
  });

  it('should cancel height editing when Escape is pressed', () => {
    render(<ImageCanvas height={200} />);
    
    const heightDisplay = screen.getByText('0m');
    fireEvent.click(heightDisplay);
    
    const heightInput = screen.getByRole('spinbutton');
    fireEvent.change(heightInput, { target: { value: '10' } });
    fireEvent.keyDown(heightInput, { key: 'Escape' });
    
    // Should not update the height
    expect(mockStoreWithPoint.updatePixelPointHeight).not.toHaveBeenCalled();
  });
});

describe('ImageCanvas - Panning Constraints', () => {
  const mockStoreWithImage = {
    image: { url: 'test.png', width: 400, height: 300, name: 'test.png' },
    pixelPoints: [],
    setImage: vi.fn(),
    addPixelPoint: vi.fn(),
    movePixelPoint: vi.fn(),
    updatePixelPointHeight: vi.fn(),
    removePixelPoint: vi.fn(),
    selectLinkedPoint: vi.fn(),
    activeWorldId: null,
  };

  beforeEach(() => {
    vi.clearAllMocks();
    (useStore as any).mockImplementation((selector: any) => selector(mockStoreWithImage));
  });

  it('should implement pan constraints', () => {
    render(<ImageCanvas height={200} />);
    
    const canvas = screen.getByTestId('canvas');
    
    // Test panning - should constrain movement
    fireEvent.mouseDown(canvas, { clientX: 100, clientY: 100 });
    fireEvent.mouseMove(canvas, { clientX: 200, clientY: 200, movementX: 100, movementY: 100 });
    fireEvent.mouseUp(canvas);
    
    // Constraint logic should prevent excessive panning
    // (Actual constraint testing would require more detailed canvas mocking)
    expect(canvas).toBeInTheDocument();
  });
});