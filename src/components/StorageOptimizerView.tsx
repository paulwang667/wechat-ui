import React, { useState, useEffect } from 'react';
import { 
  Database, 
  HardDrive, 
  Zap, 
  Layers, 
  FileCheck, 
  RefreshCw, 
  Download, 
  Upload, 
  Trash2, 
  CheckCircle2, 
  TrendingDown, 
  Sparkles,
  Server,
  Activity
} from 'lucide-react';
import { StorageMetrics } from '../types/wechat';
import { rustStorageEngine } from '../services/rustStorageEngine';

interface StorageOptimizerViewProps {
  isDarkMode: boolean;
  onRefreshMetrics: () => void;
  onStartStressTest: (count: number) => void;
}

export const StorageOptimizerView: React.FC<StorageOptimizerViewProps> = ({
  isDarkMode,
  onRefreshMetrics,
  onStartStressTest,
}) => {
  const [metrics, setMetrics] = useState<StorageMetrics>(rustStorageEngine.getMetrics());
  const [isCompacting, setIsCompacting] = useState(false);
  const [compactionResult, setCompactionResult] = useState<{ freedBytes: number; durationMs: number } | null>(null);
  const [stressCount, setStressCount] = useState<number>(2000);
  const [isStressTesting, setIsStressTesting] = useState(false);
  const [importJsonText, setImportJsonText] = useState('');
  const [importStatus, setImportStatus] = useState<string | null>(null);

  const refresh = () => {
    const m = rustStorageEngine.getMetrics();
    setMetrics(m);
    onRefreshMetrics();
  };

  useEffect(() => {
    refresh();
  }, []);

  const handleVacuumCompact = () => {
    setIsCompacting(true);
    setTimeout(() => {
      const res = rustStorageEngine.vacuumAndCompact();
      setCompactionResult({
        freedBytes: res.freedBytes,
        durationMs: res.durationMs,
      });
      setIsCompacting(false);
      refresh();
    }, 400);
  };

  const handleStressGenerate = () => {
    setIsStressTesting(true);
    setTimeout(() => {
      onStartStressTest(stressCount);
      setIsStressTesting(false);
      refresh();
    }, 300);
  };

  const handleExport = () => {
    const json = rustStorageEngine.exportDatabaseJson();
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `wechat_rust_db_backup_${Date.now()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  const handleImport = () => {
    if (!importJsonText.trim()) return;
    const res = rustStorageEngine.importDatabaseJson(importJsonText.trim());
    if (res.success) {
      setImportStatus(`Successfully restored ${res.importedCount} messages into Rust storage!`);
      setImportJsonText('');
      refresh();
    } else {
      setImportStatus(`Import error: ${res.error}`);
    }
  };

  const formatKB = (bytes: number) => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(2)} MB`;
  };

  return (
    <div
      id="storage_optimizer_view"
      className={`flex-1 flex flex-col h-full overflow-y-auto p-6 space-y-6 select-none transition-colors ${
        isDarkMode ? 'bg-[#1e1e24] text-zinc-100' : 'bg-[#f5f5f7] text-zinc-800'
      }`}
    >
      {/* Header */}
      <div className="flex items-center justify-between pb-4 border-b border-black/10 dark:border-white/10">
        <div className="flex items-center space-x-3">
          <div className="w-10 h-10 rounded-xl bg-emerald-500/10 text-emerald-500 flex items-center justify-center shadow-xs">
            <Database className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-base font-bold flex items-center space-x-2">
              <span>Local Lightweight Storage Optimizer</span>
              <span className="text-[10px] px-2 py-0.5 rounded-full bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 font-mono font-semibold">
                Rust B-Tree + LZ4 + WAL
              </span>
            </h2>
            <p className="text-xs text-zinc-400">
              Zero-copy message streaming, memory-mapped I/O, and blob deduplication.
            </p>
          </div>
        </div>

        <button
          onClick={refresh}
          title="Refresh Storage Metrics"
          className="p-2 rounded-lg hover:bg-black/5 dark:hover:bg-white/5 text-zinc-500 hover:text-emerald-500 transition-colors"
        >
          <RefreshCw className="w-4 h-4" />
        </button>
      </div>

      {/* Top 4 Metric Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Card 1: Total Messages & Convs */}
        <div className={`p-4 rounded-xl border shadow-xs space-y-2 ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span>Total Messages</span>
            <Layers className="w-4 h-4 text-emerald-500" />
          </div>
          <div className="flex items-baseline space-x-2">
            <span className="text-2xl font-bold font-mono text-zinc-900 dark:text-zinc-50">
              {metrics.totalMessages.toLocaleString()}
            </span>
            <span className="text-xs text-zinc-400 font-mono">
              in {metrics.totalConversations} chats
            </span>
          </div>
          <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
            Columnar index lookup &lt; 0.05ms
          </p>
        </div>

        {/* Card 2: Compressed Size & Compression Ratio */}
        <div className={`p-4 rounded-xl border shadow-xs space-y-2 ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span>LZ4 Compressed Store</span>
            <TrendingDown className="w-4 h-4 text-emerald-500" />
          </div>
          <div className="flex items-baseline space-x-2">
            <span className="text-2xl font-bold font-mono text-emerald-600 dark:text-emerald-400">
              {formatKB(metrics.compressedStorageBytes)}
            </span>
            <span className="text-xs font-mono text-emerald-600 font-bold bg-emerald-500/10 px-1.5 py-0.5 rounded">
              -{metrics.compressionRatio}%
            </span>
          </div>
          <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
            Raw: {formatKB(metrics.rawStorageBytes)} (Saved {formatKB(metrics.savedStorageBytes)})
          </p>
        </div>

        {/* Card 3: WAL Journal & Index */}
        <div className={`p-4 rounded-xl border shadow-xs space-y-2 ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span>WAL Journal & Index</span>
            <HardDrive className="w-4 h-4 text-blue-500" />
          </div>
          <div className="flex items-baseline space-x-2">
            <span className="text-2xl font-bold font-mono text-zinc-900 dark:text-zinc-50">
              {formatKB(metrics.walLogSizeBytes + metrics.indexSizeBytes)}
            </span>
            <span className="text-xs text-zinc-400 font-mono">
              FTS5 Tokenizer
            </span>
          </div>
          <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
            WAL: {formatKB(metrics.walLogSizeBytes)} • Index: {formatKB(metrics.indexSizeBytes)}
          </p>
        </div>

        {/* Card 4: In-Memory Ring Buffer */}
        <div className={`p-4 rounded-xl border shadow-xs space-y-2 ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <div className="flex items-center justify-between text-zinc-400 text-xs">
            <span>Hot Memory Cache</span>
            <Zap className="w-4 h-4 text-amber-500" />
          </div>
          <div className="flex items-baseline space-x-2">
            <span className="text-2xl font-bold font-mono text-zinc-900 dark:text-zinc-50">
              {formatKB(metrics.activeMemoryCacheBytes)}
            </span>
            <span className="text-xs font-mono text-emerald-500 font-bold">
              {metrics.cacheHitRatio}% Hit
            </span>
          </div>
          <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
            Zero-copy LRU active session buffer
          </p>
        </div>
      </div>

      {/* Middle Section: Storage Architecture Visualizer & Vacuum Tool */}
      <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
        {/* Storage Tier Breakdown */}
        <div className={`lg:col-span-2 p-5 rounded-xl border shadow-xs space-y-4 ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-bold flex items-center space-x-2">
              <Server className="w-4 h-4 text-emerald-500" />
              <span>Multi-Tiered Storage Layout</span>
            </h3>
            <span className="text-[11px] text-zinc-400 font-mono">
              Last compact: {new Date(metrics.lastCompactionTime).toLocaleTimeString()}
            </span>
          </div>

          {/* Storage Proportional Bar */}
          <div className="space-y-2">
            <div className="h-4 w-full rounded-full bg-zinc-200 dark:bg-zinc-700 overflow-hidden flex shadow-inner">
              <div
                style={{ width: `${Math.max(15, metrics.compressionRatio * 0.6)}%` }}
                title="LZ4 Compressed Payloads"
                className="bg-emerald-500 h-full transition-all"
              />
              <div
                style={{ width: '20%' }}
                title="B-Tree & FTS5 Indices"
                className="bg-blue-500 h-full transition-all"
              />
              <div
                style={{ width: '15%' }}
                title="SQLite WAL Journal"
                className="bg-amber-500 h-full transition-all"
              />
              <div
                style={{ width: '10%' }}
                title="Hot In-Memory Cache"
                className="bg-purple-500 h-full transition-all"
              />
            </div>

            <div className="flex flex-wrap items-center gap-4 text-xs font-mono text-zinc-500 dark:text-zinc-400">
              <div className="flex items-center space-x-1.5">
                <span className="w-2.5 h-2.5 rounded-full bg-emerald-500" />
                <span>Payloads ({formatKB(metrics.compressedStorageBytes)})</span>
              </div>
              <div className="flex items-center space-x-1.5">
                <span className="w-2.5 h-2.5 rounded-full bg-blue-500" />
                <span>Indices ({formatKB(metrics.indexSizeBytes)})</span>
              </div>
              <div className="flex items-center space-x-1.5">
                <span className="w-2.5 h-2.5 rounded-full bg-amber-500" />
                <span>WAL ({formatKB(metrics.walLogSizeBytes)})</span>
              </div>
              <div className="flex items-center space-x-1.5">
                <span className="w-2.5 h-2.5 rounded-full bg-purple-500" />
                <span>Memory Pool ({formatKB(metrics.activeMemoryCacheBytes)})</span>
              </div>
            </div>
          </div>

          {/* Architecture bullet points */}
          <div className="grid grid-cols-1 md:grid-cols-2 gap-3 pt-2 text-xs">
            <div className="p-3 rounded-lg bg-black/5 dark:bg-white/5 space-y-1">
              <span className="font-bold text-emerald-600 dark:text-emerald-400">1. Columnar Chunking & LZ4</span>
              <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
                Message payloads are compressed with LZ4 before writing to disk, reducing footprint by ~68%.
              </p>
            </div>

            <div className="p-3 rounded-lg bg-black/5 dark:bg-white/5 space-y-1">
              <span className="font-bold text-blue-600 dark:text-blue-400">2. Blob SHA-256 Deduplication</span>
              <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
                Identical images, voice notes, and file attachments share a single blob ref via content hashing.
              </p>
            </div>

            <div className="p-3 rounded-lg bg-black/5 dark:bg-white/5 space-y-1">
              <span className="font-bold text-amber-600 dark:text-amber-400">3. SQLite WAL Auto-Checkpoint</span>
              <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
                Sequential writes go into append-only WAL, completely eliminating write lock contention.
              </p>
            </div>

            <div className="p-3 rounded-lg bg-black/5 dark:bg-white/5 space-y-1">
              <span className="font-bold text-purple-600 dark:text-purple-400">4. Sub-Millisecond FTS5 Index</span>
              <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
                Inverted search index handles prefix, token, and CJK tokenization across 50k+ records instantly.
              </p>
            </div>
          </div>
        </div>

        {/* Compaction & Vacuum Action Card */}
        <div className={`p-5 rounded-xl border shadow-xs space-y-4 flex flex-col justify-between ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <div className="space-y-2">
            <h3 className="text-xs font-bold flex items-center space-x-2">
              <Sparkles className="w-4 h-4 text-emerald-500" />
              <span>Auto Vacuum & Compaction</span>
            </h3>
            <p className="text-xs text-zinc-500 dark:text-zinc-400 leading-relaxed">
              Trims fragmented pages, truncates WAL log journals, and repacks B-Tree tables into continuous disk blocks.
            </p>
          </div>

          {compactionResult && (
            <div className="p-3 rounded-lg bg-emerald-500/10 border border-emerald-500/20 text-emerald-600 dark:text-emerald-400 text-xs space-y-1 font-mono">
              <div className="flex items-center space-x-1 font-bold">
                <CheckCircle2 className="w-3.5 h-3.5" />
                <span>Compaction Completed</span>
              </div>
              <p className="text-[11px]">
                Freed: {formatKB(compactionResult.freedBytes)} in {compactionResult.durationMs}ms
              </p>
            </div>
          )}

          <button
            id="btn_vacuum_compact"
            onClick={handleVacuumCompact}
            disabled={isCompacting}
            className="w-full py-2.5 px-4 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white font-medium text-xs flex items-center justify-center space-x-2 shadow-sm transition-all disabled:opacity-50 cursor-pointer"
          >
            <RefreshCw className={`w-3.5 h-3.5 ${isCompacting ? 'animate-spin' : ''}`} />
            <span>{isCompacting ? 'Compacting Database...' : 'Run PRAGMA VACUUM'}</span>
          </button>
        </div>
      </div>

      {/* Bottom Section: Stress Generator & Backup Import/Export */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Stress Test Generator */}
        <div className={`p-5 rounded-xl border shadow-xs space-y-4 ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-bold flex items-center space-x-2">
              <Activity className="w-4 h-4 text-emerald-500" />
              <span>Storage Stress & Benchmark Generator</span>
            </h3>
          </div>
          <p className="text-xs text-zinc-500 dark:text-zinc-400">
            Generate synthetic messages in milliseconds to test local B-Tree insert throughput and virtualized UI rendering.
          </p>

          <div className="flex items-center space-x-3 text-xs">
            <label className="text-zinc-400">Count:</label>
            {[500, 2000, 5000, 10000].map((num) => (
              <button
                key={num}
                onClick={() => setStressCount(num)}
                className={`px-2.5 py-1 rounded-md font-mono text-xs transition-colors ${
                  stressCount === num
                    ? 'bg-emerald-600 text-white font-bold'
                    : 'bg-black/5 dark:bg-white/5 text-zinc-600 dark:text-zinc-300 hover:bg-black/10'
                }`}
              >
                +{num.toLocaleString()}
              </button>
            ))}
          </div>

          <button
            id="btn_run_stress_generator"
            onClick={handleStressGenerate}
            disabled={isStressTesting}
            className="w-full py-2 px-4 rounded-xl border border-emerald-500/30 text-emerald-600 dark:text-emerald-400 hover:bg-emerald-500/10 text-xs font-semibold flex items-center justify-center space-x-2 transition-colors cursor-pointer"
          >
            <Zap className="w-3.5 h-3.5" />
            <span>{isStressTesting ? 'Inserting Messages...' : `Batch Insert ${stressCount.toLocaleString()} Messages`}</span>
          </button>
        </div>

        {/* Database Backup & Restore */}
        <div className={`p-5 rounded-xl border shadow-xs space-y-4 ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <div className="flex items-center justify-between">
            <h3 className="text-xs font-bold flex items-center space-x-2">
              <FileCheck className="w-4 h-4 text-emerald-500" />
              <span>Database Backup & Snapshot</span>
            </h3>
          </div>

          <div className="flex items-center space-x-3">
            <button
              onClick={handleExport}
              className="flex-1 py-2 px-3 rounded-xl bg-zinc-200 dark:bg-zinc-700 hover:bg-zinc-300 text-zinc-800 dark:text-zinc-100 text-xs font-medium flex items-center justify-center space-x-1.5 transition-colors"
            >
              <Download className="w-3.5 h-3.5" />
              <span>Export DB Backup (.json)</span>
            </button>
          </div>

          {/* Import section */}
          <div className="space-y-2">
            <textarea
              rows={2}
              value={importJsonText}
              onChange={(e) => setImportJsonText(e.target.value)}
              placeholder="Paste JSON database snapshot to restore..."
              className={`w-full font-mono text-[11px] p-2 rounded-lg border outline-none ${
                isDarkMode ? 'bg-[#18181c] border-zinc-800 text-zinc-300' : 'bg-zinc-50 border-zinc-200 text-zinc-800'
              }`}
            />
            {importStatus && (
              <p className="text-[11px] text-emerald-500 font-mono">{importStatus}</p>
            )}
            <button
              onClick={handleImport}
              disabled={!importJsonText.trim()}
              className="w-full py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-medium disabled:opacity-40 transition-colors"
            >
              Restore Snapshot
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
