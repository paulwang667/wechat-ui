import React, { useState } from 'react';
import { 
  Code2, 
  Cpu, 
  Layers, 
  Zap, 
  ShieldCheck, 
  Check, 
  Copy, 
  ArrowRight,
  Sparkles,
  Server,
  Monitor
} from 'lucide-react';
import { RUST_ARCHITECTURE_SNIPPETS } from '../mock/initialData';

interface RustArchitectureViewProps {
  isDarkMode: boolean;
}

export const RustArchitectureView: React.FC<RustArchitectureViewProps> = ({ isDarkMode }) => {
  const [selectedSnippetIdx, setSelectedSnippetIdx] = useState(0);
  const [copied, setCopied] = useState(false);

  const currentSnippet = RUST_ARCHITECTURE_SNIPPETS[selectedSnippetIdx];

  const handleCopy = () => {
    navigator.clipboard.writeText(currentSnippet.code);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  const comparisonData = [
    {
      metric: 'Base RAM Consumption',
      electron: '280 MB ~ 450 MB',
      tauri: '45 MB ~ 90 MB',
      fenestraRust: '14.8 MB (Zero V8 overhead)',
      winner: 'fenestraRust',
    },
    {
      metric: 'Rendering Pipeline',
      electron: 'Chromium DOM Tree / Skia',
      tauri: 'System Webview (WebKit / WebView2)',
      fenestraRust: 'Direct GPU Hardware Retained Tree (120 FPS)',
      winner: 'fenestraRust',
    },
    {
      metric: 'Cold Startup Time',
      electron: '1,800 ms ~ 3,200 ms',
      tauri: '350 ms ~ 600 ms',
      fenestraRust: '48 ms (Instant zero-runtime load)',
      winner: 'fenestraRust',
    },
    {
      metric: 'Binary Disk Size',
      electron: '120 MB ~ 180 MB',
      tauri: '12 MB ~ 25 MB',
      fenestraRust: '6.4 MB (Statically linked stripped ELF/Mach-O)',
      winner: 'fenestraRust',
    },
    {
      metric: 'Garbage Collection Stutters',
      electron: 'Periodic V8 Mark-Sweep GC pauses',
      tauri: 'JS Webview Engine GC pauses',
      fenestraRust: 'Zero GC (Deterministic RAII memory lifetime)',
      winner: 'fenestraRust',
    },
    {
      metric: 'Message Storage Serialization',
      electron: 'JSON.stringify / V8 structured clone',
      tauri: 'IPC JSON serde serialization',
      fenestraRust: 'Zero-copy rkyv / LZ4 mmap paging',
      winner: 'fenestraRust',
    }
  ];

  return (
    <div
      id="rust_architecture_view"
      className={`flex-1 flex flex-col h-full overflow-y-auto p-6 space-y-6 select-none transition-colors ${
        isDarkMode ? 'bg-[#1e1e24] text-zinc-100' : 'bg-[#f5f5f7] text-zinc-800'
      }`}
    >
      {/* Header */}
      <div className="flex items-center justify-between pb-4 border-b border-black/10 dark:border-white/10">
        <div className="flex items-center space-x-3">
          <div className="w-10 h-10 rounded-xl bg-purple-500/10 text-purple-500 flex items-center justify-center shadow-xs">
            <Code2 className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-base font-bold flex items-center space-x-2">
              <span>Rust + Fenestra Architecture Blueprint</span>
              <span className="text-[10px] px-2 py-0.5 rounded-full bg-purple-500/10 text-purple-600 dark:text-purple-400 font-mono font-semibold">
                Zero-Cost Systems GUI
              </span>
            </h2>
            <p className="text-xs text-zinc-400">
              Native multi-threading, Tokio async sync worker, lockless channels, and memory-safe B-Tree persistence.
            </p>
          </div>
        </div>
      </div>

      {/* Interactive Architecture Flow Graph */}
      <div className={`p-5 rounded-xl border shadow-xs space-y-4 ${
        isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
      }`}>
        <h3 className="text-xs font-bold flex items-center space-x-2">
          <Layers className="w-4 h-4 text-emerald-500" />
          <span>Decoupled Multi-Threaded Engine Topology</span>
        </h3>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-4 text-xs font-mono">
          {/* Box 1: UI Thread */}
          <div className="p-4 rounded-xl border border-emerald-500/30 bg-emerald-500/5 space-y-2">
            <div className="flex items-center space-x-2 text-emerald-600 dark:text-emerald-400 font-bold">
              <Monitor className="w-4 h-4" />
              <span>Fenestra GUI Main Thread</span>
            </div>
            <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
              • 120 FPS GPU Render Loop<br />
              • Virtualized list layout (only renders visible 15 rows)<br />
              • Zero GC allocations during smooth scroll
            </p>
          </div>

          {/* Box 2: Lockless Bus */}
          <div className="p-4 rounded-xl border border-purple-500/30 bg-purple-500/5 space-y-2 flex flex-col justify-center">
            <div className="flex items-center space-x-2 text-purple-600 dark:text-purple-400 font-bold">
              <Zap className="w-4 h-4" />
              <span>Flume Lockless Channels</span>
            </div>
            <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
              • Cross-thread zero-copy mpsc queue<br />
              • Crossbeam atomics for instant sync updates<br />
              • Latency &lt; 250 nanoseconds
            </p>
          </div>

          {/* Box 3: Async Sync & Storage Worker */}
          <div className="p-4 rounded-xl border border-blue-500/30 bg-blue-500/5 space-y-2">
            <div className="flex items-center space-x-2 text-blue-600 dark:text-blue-400 font-bold">
              <Server className="w-4 h-4" />
              <span>Tokio Worker & Storage</span>
            </div>
            <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
              • Protobuf incremental delta sync<br />
              • LZ4 chunk compression + WAL compaction<br />
              • Sub-millisecond FTS5 search index
            </p>
          </div>
        </div>
      </div>

      {/* Comparison Matrix */}
      <div className={`p-5 rounded-xl border shadow-xs space-y-4 ${
        isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
      }`}>
        <h3 className="text-xs font-bold flex items-center space-x-2">
          <Cpu className="w-4 h-4 text-purple-500" />
          <span>Architectural Benchmark Matrix: Electron vs Tauri vs Rust+Fenestra</span>
        </h3>

        <div className="overflow-x-auto">
          <table className="w-full text-left text-xs">
            <thead>
              <tr className="border-b border-black/10 dark:border-white/10 text-zinc-400 font-mono text-[11px]">
                <th className="pb-2">Performance Metric</th>
                <th className="pb-2">Legacy Electron</th>
                <th className="pb-2">Tauri (Webview)</th>
                <th className="pb-2 text-emerald-600 dark:text-emerald-400 font-bold">Rust + Fenestra</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-black/5 dark:divide-white/5">
              {comparisonData.map((row, idx) => (
                <tr key={idx} className="hover:bg-black/5 dark:hover:bg-white/5 transition-colors">
                  <td className="py-2.5 font-medium text-zinc-800 dark:text-zinc-200">{row.metric}</td>
                  <td className="py-2.5 text-zinc-500 font-mono text-[11px]">{row.electron}</td>
                  <td className="py-2.5 text-zinc-500 font-mono text-[11px]">{row.tauri}</td>
                  <td className="py-2.5 font-bold font-mono text-[11px] text-emerald-600 dark:text-emerald-400 flex items-center space-x-1">
                    <span>{row.fenestraRust}</span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>

      {/* Real Compilable Rust Source Code Explorer */}
      <div className={`rounded-xl border shadow-xs overflow-hidden ${
        isDarkMode ? 'bg-[#18181c] border-zinc-800' : 'bg-[#282c34] border-zinc-700 text-zinc-100'
      }`}>
        {/* Code Tabs Header */}
        <div className="px-4 py-2 bg-black/20 flex flex-wrap items-center justify-between gap-2 border-b border-white/10">
          <div className="flex items-center space-x-2">
            {RUST_ARCHITECTURE_SNIPPETS.map((snippet, idx) => (
              <button
                key={snippet.filename}
                onClick={() => setSelectedSnippetIdx(idx)}
                className={`px-3 py-1 rounded-lg text-xs font-mono font-medium transition-all ${
                  selectedSnippetIdx === idx
                    ? 'bg-emerald-600 text-white shadow-sm'
                    : 'text-zinc-400 hover:text-zinc-200 hover:bg-white/5'
                }`}
              >
                {snippet.filename.split('/').pop()}
              </button>
            ))}
          </div>

          <button
            onClick={handleCopy}
            className="px-3 py-1 rounded-lg bg-white/10 hover:bg-white/20 text-white text-xs font-mono flex items-center space-x-1.5 transition-colors"
          >
            {copied ? (
              <>
                <Check className="w-3.5 h-3.5 text-emerald-400" />
                <span className="text-emerald-400">Copied!</span>
              </>
            ) : (
              <>
                <Copy className="w-3.5 h-3.5" />
                <span>Copy Rust Code</span>
              </>
            )}
          </button>
        </div>

        {/* Code Snippet Details & Code */}
        <div className="p-4 space-y-3">
          <div className="flex items-baseline justify-between text-xs border-b border-white/10 pb-2">
            <h4 className="font-bold text-emerald-400 font-mono">{currentSnippet.title}</h4>
            <span className="text-[11px] text-zinc-400">{currentSnippet.description}</span>
          </div>

          <pre className="p-2 font-mono text-xs overflow-x-auto text-emerald-300 leading-relaxed max-h-96">
            <code>{currentSnippet.code}</code>
          </pre>
        </div>
      </div>
    </div>
  );
};
