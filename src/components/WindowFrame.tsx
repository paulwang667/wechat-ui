import React from 'react';
import { 
  Minus, 
  Square, 
  X, 
  Wifi, 
  WifiOff, 
  Cpu, 
  HardDrive, 
  RefreshCw,
  Sun,
  Moon
} from 'lucide-react';
import { NetworkCondition } from '../types/wechat';

interface WindowFrameProps {
  network: NetworkCondition;
  isDarkMode: boolean;
  onToggleDarkMode: () => void;
  onManualSync: () => void;
  seqId: number;
  activeTab: string;
}

export const WindowFrame: React.FC<WindowFrameProps> = ({
  network,
  isDarkMode,
  onToggleDarkMode,
  onManualSync,
  seqId,
  activeTab,
}) => {
  return (
    <header
      id="wechat_window_titlebar"
      className={`h-9 px-3 flex items-center justify-between border-b select-none text-xs ${
        isDarkMode 
          ? 'bg-[#1e1e24] text-zinc-300 border-zinc-800' 
          : 'bg-[#f3f4f6] text-zinc-700 border-zinc-200'
      }`}
    >
      {/* Window Controls (macOS style) */}
      <div className="flex items-center space-x-2">
        <div className="flex items-center space-x-2 mr-3 group">
          <div className="w-3 h-3 rounded-full bg-[#ff5f56] border border-[#e0443e] cursor-pointer flex items-center justify-center opacity-90 hover:opacity-100">
            <X className="w-2 h-2 text-[#4c0000] opacity-0 group-hover:opacity-100 transition-opacity" />
          </div>
          <div className="w-3 h-3 rounded-full bg-[#ffbd2e] border border-[#dea123] cursor-pointer flex items-center justify-center opacity-90 hover:opacity-100">
            <Minus className="w-2 h-2 text-[#5a3b00] opacity-0 group-hover:opacity-100 transition-opacity" />
          </div>
          <div className="w-3 h-3 rounded-full bg-[#27c93f] border border-[#1aab29] cursor-pointer flex items-center justify-center opacity-90 hover:opacity-100">
            <Square className="w-1.5 h-1.5 text-[#004d11] opacity-0 group-hover:opacity-100 transition-opacity" />
          </div>
        </div>

        <div className="flex items-center space-x-2 font-medium">
          <span className="font-semibold text-emerald-600 dark:text-emerald-400">微信 WeChat</span>
          <span className="text-[10px] px-1.5 py-0.5 rounded bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 font-mono font-bold">
            Rust + Fenestra Native
          </span>
        </div>
      </div>

      {/* Center status badge */}
      <div className="hidden md:flex items-center space-x-4 text-[11px] text-zinc-500 dark:text-zinc-400">
        <div className="flex items-center space-x-1.5 font-mono">
          <Cpu className="w-3.5 h-3.5 text-emerald-500" />
          <span>RAM: <strong className="text-zinc-700 dark:text-zinc-200">14.8 MB</strong></span>
          <span className="text-zinc-400">|</span>
          <span>FPS: <strong className="text-zinc-700 dark:text-zinc-200">120</strong></span>
        </div>

        <div className="flex items-center space-x-1.5 font-mono">
          <HardDrive className="w-3.5 h-3.5 text-blue-500" />
          <span>Sync Seq: <strong className="text-zinc-700 dark:text-zinc-200">#{seqId}</strong></span>
        </div>
      </div>

      {/* Right Controls */}
      <div className="flex items-center space-x-3">
        {/* Network status pill */}
        <div 
          title={`Network Mode: ${network.mode.toUpperCase()} (Latency: ${network.latencyMs}ms)`}
          className={`flex items-center space-x-1 px-2 py-0.5 rounded-full text-[10px] font-medium font-mono ${
            network.mode === 'offline'
              ? 'bg-rose-500/10 text-rose-500 border border-rose-500/20'
              : network.mode === 'packet_loss' || network.mode === 'high_latency'
              ? 'bg-amber-500/10 text-amber-500 border border-amber-500/20'
              : 'bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20'
          }`}
        >
          {network.mode === 'offline' ? (
            <WifiOff className="w-3 h-3 animate-pulse" />
          ) : (
            <Wifi className="w-3 h-3" />
          )}
          <span className="capitalize">{network.mode.replace('_', ' ')}</span>
        </div>

        {/* Sync Trigger */}
        <button
          id="btn_manual_sync_trigger"
          onClick={onManualSync}
          title="Trigger Incremental Sequence Pull"
          className="p-1 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-600 dark:text-zinc-300 transition-colors"
        >
          <RefreshCw className="w-3.5 h-3.5" />
        </button>

        {/* Theme Toggle */}
        <button
          id="btn_theme_toggle"
          onClick={onToggleDarkMode}
          title={isDarkMode ? 'Switch to Light Mode' : 'Switch to Dark Mode'}
          className="p-1 rounded hover:bg-zinc-200 dark:hover:bg-zinc-700 text-zinc-600 dark:text-zinc-300 transition-colors"
        >
          {isDarkMode ? <Sun className="w-3.5 h-3.5" /> : <Moon className="w-3.5 h-3.5" />}
        </button>
      </div>
    </header>
  );
};
