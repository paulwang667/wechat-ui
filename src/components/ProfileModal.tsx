import React, { useState } from 'react';
import { 
  X, 
  User, 
  Settings, 
  ShieldCheck, 
  HardDrive, 
  Cpu, 
  Zap, 
  Check, 
  RotateCcw
} from 'lucide-react';
import { Contact } from '../types/wechat';

interface ProfileModalProps {
  isOpen: boolean;
  onClose: () => void;
  currentUser: Contact;
  onUpdateUser: (user: Partial<Contact>) => void;
  isDarkMode: boolean;
}

export const ProfileModal: React.FC<ProfileModalProps> = ({
  isOpen,
  onClose,
  currentUser,
  onUpdateUser,
  isDarkMode,
}) => {
  const [name, setName] = useState(currentUser.name);
  const [bio, setBio] = useState(currentUser.bio);
  const [alias, setAlias] = useState(currentUser.alias || '');
  const [compressionEnabled, setCompressionEnabled] = useState(true);
  const [autoVacuumInterval, setAutoVacuumInterval] = useState('hourly');
  const [savedSuccess, setSavedSuccess] = useState(false);

  if (!isOpen) return null;

  const handleSave = () => {
    onUpdateUser({ name, bio, alias });
    setSavedSuccess(true);
    setTimeout(() => {
      setSavedSuccess(false);
      onClose();
    }, 800);
  };

  return (
    <div className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center p-4 z-50 select-none">
      <div className={`w-full max-w-md rounded-2xl border shadow-2xl overflow-hidden transition-colors ${
        isDarkMode ? 'bg-[#1e1e24] border-zinc-800 text-zinc-100' : 'bg-white border-zinc-200 text-zinc-800'
      }`}>
        {/* Header */}
        <div className="px-5 py-4 border-b border-black/10 dark:border-white/10 flex items-center justify-between">
          <div className="flex items-center space-x-2">
            <Settings className="w-4 h-4 text-emerald-500" />
            <h3 className="text-sm font-bold">Profile & Engine Settings</h3>
          </div>
          <button onClick={onClose} className="p-1 rounded-lg text-zinc-400 hover:text-zinc-600">
            <X className="w-4 h-4" />
          </button>
        </div>

        {/* Content */}
        <div className="p-5 space-y-5 text-xs">
          {/* User Avatar & Name */}
          <div className="flex items-center space-x-4">
            <img
              src={currentUser.avatar}
              alt={currentUser.name}
              className="w-14 h-14 rounded-xl object-cover ring-2 ring-emerald-500/40 shadow-sm"
            />
            <div className="flex-1 space-y-1">
              <input
                type="text"
                value={name}
                onChange={(e) => setName(e.target.value)}
                placeholder="Your Display Name"
                className={`w-full px-2.5 py-1.5 rounded-lg border text-xs font-semibold outline-none ${
                  isDarkMode ? 'bg-[#18181c] border-zinc-700 text-zinc-100' : 'bg-zinc-50 border-zinc-300'
                }`}
              />
              <p className="text-[11px] text-zinc-400 font-mono">
                ID: {currentUser.wechatId}
              </p>
            </div>
          </div>

          {/* Bio input */}
          <div className="space-y-1">
            <label className="text-zinc-400 font-medium">Signature / Status</label>
            <textarea
              rows={2}
              value={bio}
              onChange={(e) => setBio(e.target.value)}
              className={`w-full px-2.5 py-1.5 rounded-lg border text-xs outline-none ${
                isDarkMode ? 'bg-[#18181c] border-zinc-700 text-zinc-200' : 'bg-zinc-50 border-zinc-300'
              }`}
            />
          </div>

          {/* Rust Engine Tuning Settings */}
          <div className="space-y-3 pt-2 border-t border-black/5 dark:border-white/5">
            <h4 className="font-bold flex items-center space-x-1.5 text-zinc-700 dark:text-zinc-300">
              <Cpu className="w-3.5 h-3.5 text-emerald-500" />
              <span>Rust Storage & Sync Engine Parameters</span>
            </h4>

            <div className="flex items-center justify-between">
              <span className="text-zinc-500 dark:text-zinc-400">LZ4 Block Compression:</span>
              <button
                onClick={() => setCompressionEnabled(!compressionEnabled)}
                className={`px-2 py-0.5 rounded font-mono text-[11px] font-bold ${
                  compressionEnabled ? 'bg-emerald-500/10 text-emerald-500' : 'bg-zinc-500/10 text-zinc-400'
                }`}
              >
                {compressionEnabled ? 'ENABLED (68% saving)' : 'DISABLED'}
              </button>
            </div>

            <div className="flex items-center justify-between">
              <span className="text-zinc-500 dark:text-zinc-400">WAL Auto-Checkpoint:</span>
              <select
                value={autoVacuumInterval}
                onChange={(e) => setAutoVacuumInterval(e.target.value)}
                className={`px-2 py-1 rounded border text-[11px] outline-none ${
                  isDarkMode ? 'bg-[#18181c] border-zinc-700 text-zinc-200' : 'bg-zinc-100 border-zinc-300'
                }`}
              >
                <option value="realtime">Every 1,000 writes</option>
                <option value="hourly">Hourly Truncate</option>
                <option value="manual">Manual PRAGMA VACUUM</option>
              </select>
            </div>

            <div className="flex items-center justify-between text-zinc-500 dark:text-zinc-400">
              <span>Zero-Copy In-Memory Buffer:</span>
              <span className="font-mono text-emerald-500 font-bold">1,024 Slots RingBuffer</span>
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="px-5 py-3 border-t border-black/10 dark:border-white/10 flex items-center justify-end space-x-2">
          <button
            onClick={onClose}
            className="px-3.5 py-1.5 rounded-xl border border-zinc-300 dark:border-zinc-700 hover:bg-black/5 text-xs transition-colors"
          >
            Cancel
          </button>
          <button
            onClick={handleSave}
            className="px-4 py-1.5 rounded-xl bg-emerald-600 hover:bg-emerald-500 text-white font-medium text-xs flex items-center space-x-1.5 shadow-sm transition-all"
          >
            {savedSuccess ? (
              <>
                <Check className="w-3.5 h-3.5" />
                <span>Saved</span>
              </>
            ) : (
              <span>Save Changes</span>
            )}
          </button>
        </div>
      </div>
    </div>
  );
};
