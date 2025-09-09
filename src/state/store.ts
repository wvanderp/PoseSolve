import { create } from 'zustand';

export type PixelPoint = { id: string; u: number; v: number; sigmaPx?: number; enabled?: boolean };
export type WorldPoint = { id: string; lat: number; lon: number; alt?: number; sigmaM?: number };
export type Link = { pixelId: string; worldId: string };

export type ImageInfo = { url?: string; width: number; height: number; name?: string };

type State = {
  image: ImageInfo | null;
  pixelPoints: PixelPoint[];
  worldPoints: WorldPoint[];
  links: Link[];
  activePixelId: string | null;
  activeWorldId: string | null;
  // actions
  setImage: (img: ImageInfo | null) => void;
  addPixelPoint: (p: Omit<PixelPoint, 'id'> & { id?: string }) => string;
  movePixelPoint: (id: string, u: number, v: number) => void;
  removePixelPoint: (id: string) => void;
  addWorldPoint: (p: Omit<WorldPoint, 'id'> & { id?: string }) => string;
  moveWorldPoint: (id: string, lat: number, lon: number) => void;
  removeWorldPoint: (id: string) => void;
  linkPoints: (pixelId: string, worldId: string) => void;
  unlinkByPixel: (pixelId: string) => void;
  setActivePixel: (id: string | null) => void;
  setActiveWorld: (id: string | null) => void;
};

let _idCounter = 0;
const genId = (prefix: string) => `${prefix}_${Date.now().toString(36)}_${(_idCounter++).toString(36)}`;

export const useStore = create<State>((set, get) => ({
  // Default image (requested): Coolhaven / Erasmus medical center Rotterdam skyline
  image: {
    url: 'https://upload.wikimedia.org/wikipedia/commons/9/9e/Coolhaven_Erasmus_medical_center_Rotterdam_skyline.jpg',
    // placeholder dimensions; ImageCanvas will update these to the real natural size when the image loads
    width: 1,
    height: 1,
    name: 'Coolhaven Rotterdam (default)'
  },
  pixelPoints: [],
  worldPoints: [],
  links: [],
  activePixelId: null,
  activeWorldId: null,
  setImage: (img) => set({ image: img }),
  addPixelPoint: (p) => {
    const id = p.id ?? genId('px');
    set((s) => ({ pixelPoints: [...s.pixelPoints, { id, enabled: true, sigmaPx: 1, ...p }] }));
    return id;
  },
  movePixelPoint: (id, u, v) => set((s) => ({
    pixelPoints: s.pixelPoints.map(pp => pp.id === id ? { ...pp, u, v } : pp)
  })),
  removePixelPoint: (id) => set((s) => ({
    pixelPoints: s.pixelPoints.filter(pp => pp.id !== id),
    links: s.links.filter(l => l.pixelId !== id),
    activePixelId: s.activePixelId === id ? null : s.activePixelId,
  })),
  addWorldPoint: (p) => {
    const id = p.id ?? genId('w');
    set((s) => ({ worldPoints: [...s.worldPoints, { id, ...p }] }));
    return id;
  },
  moveWorldPoint: (id, lat, lon) => set((s) => ({
    worldPoints: s.worldPoints.map(wp => wp.id === id ? { ...wp, lat, lon } : wp)
  })),
  removeWorldPoint: (id) => set((s) => ({
    worldPoints: s.worldPoints.filter(wp => wp.id !== id),
    links: s.links.filter(l => l.worldId !== id),
    activeWorldId: s.activeWorldId === id ? null : s.activeWorldId,
  })),
  linkPoints: (pixelId, worldId) => set((s) => {
    const exists = s.links.some(l => l.pixelId === pixelId || l.worldId === worldId);
    // Enforce 1-1 linking by removing existing links for either side
    const filtered = s.links.filter(l => l.pixelId !== pixelId && l.worldId !== worldId);
    return { links: [...filtered, { pixelId, worldId }], activePixelId: null, activeWorldId: null };
  }),
  unlinkByPixel: (pixelId) => set((s) => ({ links: s.links.filter(l => l.pixelId !== pixelId) })),
  setActivePixel: (id) => set({ activePixelId: id }),
  setActiveWorld: (id) => set({ activeWorldId: id }),
}));

export const selectors = {
  getLinkedWorldForPixel: (state: State, pixelId: string): WorldPoint | null => {
    const link = state.links.find(l => l.pixelId === pixelId);
    if (!link) return null;
    return state.worldPoints.find(w => w.id === link.worldId) ?? null;
  },
  getLinkedPixelForWorld: (state: State, worldId: string): PixelPoint | null => {
    const link = state.links.find(l => l.worldId === worldId);
    if (!link) return null;
    return state.pixelPoints.find(p => p.id === link.pixelId) ?? null;
  }
};
