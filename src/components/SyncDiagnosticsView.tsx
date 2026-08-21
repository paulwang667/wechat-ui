import React, { useState, useEffect } from 'react';
import { 
  Activity, 
  Wifi, 
  WifiOff, 
  RefreshCw, 
  Send, 
  Inbox, 
  AlertTriangle, 
  CheckCircle2, 
  ArrowDownUp, 
  Radio, 
  Clock, 
  Layers,
  Terminal,
  Trash2
} from 'lucide-react';
import { NetworkCondition, SyncLogEntry, Message } from '../types/wechat';
import { syncEngine } from '../services/syncEngine';

interface SyncDiagnosticsViewProps {
  isDarkMode: boolean;
  network: NetworkCondition;
  onNetworkChange: (cond: Partial<NetworkCondition>) => void;
  onTriggerManualSync: () => void;
}

export const SyncDiagnosticsView: React.FC<SyncDiagnosticsViewProps> = ({
  isDarkMode,
  network,
  onNetworkChange,
  onTriggerManualSync,
}) => {
  const [logs, setLogs] = useState<SyncLogEntry[]>(syncEngine.getSyncLogs());
  const [outbox, setOutbox] = useState<Message[]>(syncEngine.getOutbox());
  const [seqId, setSeqId] = useState<number>(syncEngine.getCurrentSeq());
  const [logFilter, setLogFilter] = useState<string>('ALL');

  useEffect(() => {
    const unsub = syncEngine.subscribe((event) => {
      if (event.type === 'log_added' || event.type === 'network_changed' || event.type === 'message_status_changed' || event.type === 'sync_completed') {
        setLogs(syncEngine.getSyncLogs());
        setOutbox(syncEngine.getOutbox());
        setSeqId(syncEngine.getCurrentSeq());
      }
    });

    const interval = setInterval(() => {
      setLogs(syncEngine.getSyncLogs());
      setOutbox(syncEngine.getOutbox());
      setSeqId(syncEngine.getCurrentSeq());
    }, 1500);

    return () => {
      unsub();
      clearInterval(interval);
    };
  }, []);

  const networkProfiles: {
    id: NetworkCondition['mode'];
    label: string;
    latency: number;
    loss: number;
    desc: string;
  }[] = [
    { id: 'optimal', label: 'Optimal LAN', latency: 8, loss: 0, desc: 'Ultra-low latency, 0% packet loss' },
    { id: 'wifi', label: 'Standard Wi-Fi', latency: 25, loss: 0, desc: 'Typical desktop office connection' },
    { id: 'cellular_4g', label: 'Cellular 4G', latency: 90, loss: 0.01, desc: 'Moderate latency, occasional jitter' },
    { id: 'high_latency', label: 'High Latency (450ms)', latency: 450, loss: 0.02, desc: 'Cross-border / satellite link' },
    { id: 'packet_loss', label: 'Flaky Packet Loss (25%)', latency: 80, loss: 0.25, desc: 'Simulates dropped ACKs & Tokio auto-retry' },
    { id: 'offline', label: 'Offline / Airplane', latency: 9999, loss: 1.0, desc: 'Messages held in Outbox queue' },
  ];

  const filteredLogs = logs.filter((log) => {
    if (logFilter === 'ALL') return true;
    return log.type === logFilter;
  });

  return (
    <div
      id="sync_diagnostics_view"
      className={`flex-1 flex flex-col h-full overflow-y-auto p-6 space-y-6 select-none transition-colors ${
        isDarkMode ? 'bg-[#1e1e24] text-zinc-100' : 'bg-[#f5f5f7] text-zinc-800'
      }`}
    >
      {/* Header */}
      <div className="flex items-center justify-between pb-4 border-b border-black/10 dark:border-white/10">
        <div className="flex items-center space-x-3">
          <div className="w-10 h-10 rounded-xl bg-blue-500/10 text-blue-500 flex items-center justify-center shadow-xs">
            <ArrowDownUp className="w-5 h-5" />
          </div>
          <div>
            <h2 className="text-base font-bold flex items-center space-x-2">
              <span>Incremental Sequence Sync Protocol Engine</span>
              <span className="text-[10px] px-2 py-0.5 rounded-full bg-blue-500/10 text-blue-600 dark:text-blue-400 font-mono font-semibold">
                Protobuf Delta Sync
              </span>
            </h2>
            <p className="text-xs text-zinc-400">
              Monotonic sequence ordering, vector clock convergence, and Tokio async outbox dispatcher.
            </p>
          </div>
        </div>

        <button
          id="btn_trigger_sync_pull"
          onClick={onTriggerManualSync}
          className="px-3.5 py-1.5 rounded-xl bg-blue-600 hover:bg-blue-500 text-white text-xs font-semibold flex items-center space-x-1.5 shadow-sm transition-all cursor-pointer"
        >
          <RefreshCw className="w-3.5 h-3.5" />
          <span>Pull Delta Sync</span>
        </button>
      </div>

      {/* Sequence & Outbox Status Cards */}
      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
        <div className={`p-4 rounded-xl border shadow-xs space-y-1.5 ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <span className="text-xs text-zinc-400">Monotonic Sequence ID</span>
          <div className="text-2xl font-bold font-mono text-blue-500">
            #{seqId}
          </div>
          <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
            Strict total ordering invariant
          </p>
        </div>

        <div className={`p-4 rounded-xl border shadow-xs space-y-1.5 ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <span className="text-xs text-zinc-400">Outbox Pending Queue</span>
          <div className="text-2xl font-bold font-mono text-zinc-900 dark:text-zinc-50">
            {outbox.length}
          </div>
          <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
            {outbox.length === 0 ? 'All messages ACKed' : 'Waiting for network sync'}
          </p>
        </div>

        <div className={`p-4 rounded-xl border shadow-xs space-y-1.5 ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <span className="text-xs text-zinc-400">Current RTT Latency</span>
          <div className="text-2xl font-bold font-mono text-emerald-500">
            {network.mode === 'offline' ? '∞ ms' : `${network.latencyMs} ms`}
          </div>
          <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
            Loss rate: {(network.packetLossRate * 100).toFixed(0)}%
          </p>
        </div>

        <div className={`p-4 rounded-xl border shadow-xs space-y-1.5 ${
          isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
        }`}>
          <span className="text-xs text-zinc-400">Sync Channel Mode</span>
          <div className="text-sm font-bold font-mono text-zinc-900 dark:text-zinc-50 uppercase mt-1">
            {network.mode.replace('_', ' ')}
          </div>
          <p className="text-[11px] text-zinc-500 dark:text-zinc-400">
            Tokio Async Worker Active
          </p>
        </div>
      </div>

      {/* Interactive Network Simulator Switch */}
      <div className={`p-5 rounded-xl border shadow-xs space-y-3 ${
        isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
      }`}>
        <div className="flex items-center justify-between">
          <h3 className="text-xs font-bold flex items-center space-x-2">
            <Radio className="w-4 h-4 text-blue-500" />
            <span>Network Condition Simulator (Test Sync Resilience & Auto-Retry)</span>
          </h3>
        </div>

        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3 pt-1">
          {networkProfiles.map((p) => {
            const isCurrent = network.mode === p.id;
            return (
              <button
                key={p.id}
                id={`network_mode_${p.id}`}
                onClick={() => onNetworkChange({
                  mode: p.id,
                  latencyMs: p.latency,
                  packetLossRate: p.loss,
                })}
                className={`p-3 rounded-xl border text-left flex flex-col justify-between transition-all cursor-pointer ${
                  isCurrent
                    ? 'border-blue-500 bg-blue-500/10 ring-2 ring-blue-500/20'
                    : 'border-zinc-200 dark:border-zinc-700/60 hover:bg-black/5 dark:hover:bg-white/5'
                }`}
              >
                <div className="flex items-center justify-between">
                  <span className="text-xs font-bold text-zinc-900 dark:text-zinc-100">{p.label}</span>
                  {p.id === 'offline' ? (
                    <WifiOff className="w-3.5 h-3.5 text-rose-500" />
                  ) : (
                    <Wifi className={`w-3.5 h-3.5 ${isCurrent ? 'text-blue-500' : 'text-zinc-400'}`} />
                  )}
                </div>
                <p className="text-[11px] text-zinc-500 dark:text-zinc-400 mt-1.5 leading-tight">
                  {p.desc}
                </p>
                <div className="mt-2 text-[10px] font-mono text-zinc-400 flex items-center justify-between border-t border-black/5 dark:border-white/5 pt-1.5">
                  <span>RTT: {p.latency === 9999 ? 'Offline' : `${p.latency}ms`}</span>
                  <span>Loss: {(p.loss * 100).toFixed(0)}%</span>
                </div>
              </button>
            );
          })}
        </div>
      </div>

      {/* Outbox Queue Section if not empty */}
      {outbox.length > 0 && (
        <div className={`p-4 rounded-xl border border-amber-500/30 bg-amber-500/5 space-y-3`}>
          <div className="flex items-center justify-between">
            <h4 className="text-xs font-bold text-amber-600 dark:text-amber-400 flex items-center space-x-2">
              <AlertTriangle className="w-4 h-4" />
              <span>Outbox Queue ({outbox.length} pending unacked messages)</span>
            </h4>
            <button
              onClick={() => syncEngine.flushOutbox()}
              className="px-2.5 py-1 rounded bg-amber-600 hover:bg-amber-500 text-white text-[11px] font-semibold transition-colors"
            >
              Force Flush Outbox
            </button>
          </div>
          <div className="space-y-1.5">
            {outbox.map((msg) => (
              <div key={msg.id} className="p-2 rounded bg-black/5 dark:bg-white/5 flex items-center justify-between text-xs font-mono">
                <span className="truncate flex-1 pr-2">Seq #{msg.seqId}: {msg.content}</span>
                <span className="text-amber-500 font-bold uppercase text-[10px]">[{msg.status}]</span>
              </div>
            ))}
          </div>
        </div>
      )}

      {/* Real-time Sync Protocol Timeline Logs */}
      <div className={`p-5 rounded-xl border shadow-xs space-y-4 ${
        isDarkMode ? 'bg-[#25252b] border-zinc-800' : 'bg-white border-zinc-200'
      }`}>
        <div className="flex flex-col sm:flex-row sm:items-center justify-between gap-2 border-b border-black/5 dark:border-white/5 pb-3">
          <div className="flex items-center space-x-2">
            <Terminal className="w-4 h-4 text-blue-500" />
            <h3 className="text-xs font-bold">Sync Protocol Event Logs</h3>
          </div>

          <div className="flex items-center space-x-1 text-[11px]">
            {['ALL', 'PUSH_MSG', 'PULL_REQ', 'PULL_RESP', 'ACK', 'HEARTBEAT'].map((f) => (
              <button
                key={f}
                onClick={() => setLogFilter(f)}
                className={`px-2 py-0.5 rounded font-mono text-[10px] transition-colors ${
                  logFilter === f
                    ? 'bg-blue-600 text-white font-bold'
                    : 'text-zinc-400 hover:bg-black/5 dark:hover:bg-white/5'
                }`}
              >
                {f}
              </button>
            ))}
          </div>
        </div>

        {/* Log List */}
        <div className="max-h-80 overflow-y-auto space-y-1.5 font-mono text-[11px]">
          {filteredLogs.length === 0 ? (
            <div className="py-8 text-center text-zinc-400 text-xs">
              No logs for selected filter.
            </div>
          ) : (
            filteredLogs.map((log) => {
              const typeColor = 
                log.type === 'ACK' ? 'text-emerald-500 bg-emerald-500/10' :
                log.type === 'PUSH_MSG' ? 'text-blue-500 bg-blue-500/10' :
                log.type === 'PULL_RESP' ? 'text-purple-500 bg-purple-500/10' :
                log.type === 'PULL_REQ' ? 'text-amber-500 bg-amber-500/10' :
                'text-zinc-400 bg-zinc-500/10';

              return (
                <div
                  key={log.id}
                  className={`p-2 rounded flex items-start space-x-2.5 transition-colors ${
                    isDarkMode ? 'bg-[#1a1a20] hover:bg-[#202028]' : 'bg-zinc-50 hover:bg-zinc-100'
                  }`}
                >
                  <span className="text-zinc-400 text-[10px] shrink-0 mt-0.5">
                    {new Date(log.timestamp).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })}
                  </span>

                  <span className={`px-1.5 py-0.2 rounded text-[9px] font-bold shrink-0 ${typeColor}`}>
                    {log.type}
                  </span>

                  <span className="text-zinc-400 text-[10px] shrink-0">
                    Seq #{log.seq}
                  </span>

                  <p className="flex-1 text-zinc-800 dark:text-zinc-200 truncate">
                    {log.details}
                  </p>

                  <span className="text-zinc-400 text-[10px] shrink-0">
                    {log.latencyMs}ms ({log.bytesTransferred}B)
                  </span>
                </div>
              );
            })
          )}
        </div>
      </div>
    </div>
  );
};
